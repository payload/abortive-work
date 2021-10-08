use std::f32::consts::TAU;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use lyon::{
    geom::euclid::point2,
    lyon_tessellation::{
        BuffersBuilder, StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers,
    },
    path::Path,
};

use crate::extensions::ToPoint;

pub fn disk(radius: f32, segments: u16) -> Mesh {
    let mut indices = Vec::<u16>::new();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let a = -(i as f32 / segments as f32) * TAU;
        let x = a.cos() * radius;
        let z = a.sin() * radius;
        let u = (a.cos() + 1.0) * 0.5;
        let v = (a.sin() + 1.0) * 0.5;

        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([u, v]);
    }

    for i in 1..=segments {
        indices.push(0);
        indices.push(i);
        indices.push(if i == segments { 1 } else { i + 1 });
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub fn cone(radius: f32, height: f32, segments: u16) -> Mesh {
    let mut indices = Vec::<u16>::new();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    positions.push([0.0, height, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let a = -(i as f32 / segments as f32) * TAU;
        let x = a.cos() * radius;
        let z = a.sin() * radius;
        let u = (a.cos() + 1.0) * 0.5;
        let v = (a.cos() + 1.0) * 0.5;

        let normal = Vec3::new(a.cos(), height, a.sin()).normalize_or_zero();

        positions.push([x, 0.0, z]);
        normals.push(normal.to_array());
        uvs.push([u, v]);
    }

    for i in 1..=segments {
        indices.push(0);
        indices.push(i);
        indices.push(if i == segments { 1 } else { i + 1 });
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub fn ring(outer_radius: f32, inner_radius: f32, segments: u16) -> Mesh {
    let mut indices = Vec::<u16>::new();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let segments_f = segments as f32;

    for i in 0..segments {
        let i2 = (i + 1) % segments;
        let ia = i as f32;
        let ib = i2 as f32;
        let a = -(ia / segments_f) * TAU;
        let b = -(ib / segments_f) * TAU;
        let [ax, az, bx, bz] = [a.cos(), a.sin(), b.cos(), b.sin()];
        let u = (ax + 1.0) * 0.5;
        let v = (az + 1.0) * 0.5;

        // NOTE could produce less vertices here if needed
        positions.push([ax * outer_radius, 0.0, az * outer_radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([u, v]);

        positions.push([bx * outer_radius, 0.0, bz * outer_radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([u, v]);

        positions.push([ax * inner_radius, 0.0, az * inner_radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([u, v]);

        let j = i * 3;
        indices.push(j);
        indices.push(j + 1);
        indices.push(j + 2);

        let j2 = i2 * 3;
        indices.push(j + 1);
        indices.push(j2 + 2);
        indices.push(j + 2);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub fn curve(start: Vec3, mid: Vec3, end: Vec3, width: f32) -> Mesh {
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let start = (start - mid).to_point();
    let end = (end - mid).to_point();

    let mut builder = Path::builder();
    builder.begin(start);
    builder.line_to(start * 0.9);
    builder.cubic_bezier_to(start * 0.5, end * 0.5, end * 0.9);
    builder.line_to(end);
    builder.end(false);
    let path = builder.build();

    let mut vertex_buffer: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
    let mut tesselator = StrokeTessellator::new();
    tesselator
        .tessellate_path(
            &path,
            &StrokeOptions::default()
                .with_line_width(width)
                .with_tolerance(0.01),
            &mut BuffersBuilder::new(&mut vertex_buffer, |vertex: StrokeVertex| {
                [vertex.position().x, 0.0, vertex.position().y]
            }),
        )
        .unwrap();

    normals.extend(vertex_buffer.vertices.iter().map(|_| [0.0, 1.0, 0.0]));
    uvs.extend(vertex_buffer.vertices.iter().map(|_| [0.0, 0.0]));

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(vertex_buffer.indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertex_buffer.vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub fn triangle() -> Mesh {
    let mut builder = Path::builder();
    builder.begin(point2(0.5, 0.0));
    builder.line_to(point2(-0.5, 0.0));
    builder.line_to(point2(0.0, 1.0));
    builder.end(true);
    let path = builder.build();

    let mut vertex_buffer: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
    let mut tesselator = StrokeTessellator::new();
    tesselator
        .tessellate_path(
            &path,
            &StrokeOptions::default()
                .with_line_width(0.1)
                .with_tolerance(0.01),
            &mut BuffersBuilder::new(&mut vertex_buffer, |vertex: StrokeVertex| {
                [vertex.position().x, 0.0, vertex.position().y]
            }),
        )
        .unwrap();

    let normals = vertex_buffer
        .vertices
        .iter()
        .map(|_| [0.0, 1.0, 0.0])
        .collect::<Vec<_>>();
    let uvs = vertex_buffer
        .vertices
        .iter()
        .map(|_| [0.0, 0.0])
        .collect::<Vec<_>>();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(vertex_buffer.indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertex_buffer.vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

#[allow(unused)]
pub fn mesh_randomize(mesh: &mut Mesh, radius: f32) {
    let r = || -0.5 + radius * fastrand::f32();
    if let Some(VertexAttributeValues::Float32x3(vecs)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for vec in vecs.iter_mut() {
            vec[0] += r();
            vec[1] += r();
            vec[2] += r();
        }
    }
}
