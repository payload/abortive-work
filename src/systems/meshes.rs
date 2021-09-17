use std::f32::consts::TAU;

use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};

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
