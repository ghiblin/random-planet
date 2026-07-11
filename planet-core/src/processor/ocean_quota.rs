use std::fmt;

use crate::geometry::mesh::{Mesh, MeshError, Vertex};

const DEFAULT_OCEAN_QUOTA: f32 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OceanQuota(pub(crate) f32);

#[derive(Debug, Clone, PartialEq)]
pub enum OceanQuotaError {
    OutOfRange { value: f32 },
}

impl fmt::Display for OceanQuotaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OceanQuotaError::OutOfRange { value } => {
                write!(f, "ocean quota must be between 0.0 and 1.0, got {value}")
            }
        }
    }
}

impl std::error::Error for OceanQuotaError {}

impl OceanQuota {
    pub fn new(value: f32) -> Result<OceanQuota, OceanQuotaError> {
        if (0.0..=1.0).contains(&value) {
            Ok(OceanQuota(value))
        } else {
            Err(OceanQuotaError::OutOfRange { value })
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for OceanQuota {
    fn default() -> Self {
        OceanQuota(DEFAULT_OCEAN_QUOTA)
    }
}

pub fn apply_ocean_quota(mesh: &Mesh, quota: OceanQuota) -> Result<Mesh, MeshError> {
    let mut radii: Vec<f32> = mesh
        .vertices()
        .iter()
        .map(|v| v.position.length())
        .collect();
    if radii.is_empty() {
        return Ok(mesh.clone());
    }
    radii.sort_by(f32::total_cmp);
    let index = ((quota.value() * radii.len() as f32) as usize).min(radii.len() - 1);
    let sea_level = radii[index];

    let vertices = mesh
        .vertices()
        .iter()
        .map(|vertex| {
            let radius = vertex.position.length();
            if radius < sea_level {
                match vertex.position.normalized() {
                    Some(direction) => Vertex {
                        position: direction.scale(sea_level),
                    },
                    None => *vertex,
                }
            } else {
                *vertex
            }
        })
        .collect();

    Mesh::new(vertices, mesh.triangles().to_vec())
}
