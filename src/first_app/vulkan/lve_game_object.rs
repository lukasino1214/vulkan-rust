use super::lve_model::*;

use std::rc::Rc;

extern crate nalgebra as na;

pub struct TransformComponent {
    pub translation: na::Vector3<f32>,
    pub scale: na::Vector3<f32>,
    pub rotation: na::Vector3<f32>,
}

impl TransformComponent {
    #[allow(dead_code)]
    pub fn mat4(&self) -> na::Matrix4<f32> {

        let c3 = self.rotation[2].cos();
        let s3 = self.rotation[2].sin();
        let c2 = self.rotation[0].cos();
        let s2 = self.rotation[0].sin();
        let c1 = self.rotation[1].cos();
        let s1 = self.rotation[1].sin();

        na::matrix!(self.scale[0] * (c1 * c3 + s1 * s2 * s3), self.scale[1] * (c3 * s1 * s2 - c1 * s3), self.scale[2] * (c2 * s1), self.translation[0];
                    self.scale[0] * (c2 * s3)               , self.scale[1] * (c2 * c3)                , self.scale[2] * (-s2)    , self.translation[1];
                    self.scale[0] * (c1 * s2 * s3 - c3 * s1), self.scale[1] * (c1 * c3 * s2 + s1 * s3), self.scale[2] * (c1 * c2), self.translation[2];
                    0.0                                     , 0.0                                     , 0.0                      , 1.0;
                )
    }

    #[allow(dead_code)]
    pub fn normal_matrix(&self) -> na::Matrix4<f32> {

        let c3 = self.rotation[2].cos();
        let s3 = self.rotation[2].sin();
        let c2 = self.rotation[0].cos();
        let s2 = self.rotation[0].sin();
        let c1 = self.rotation[1].cos();
        let s1 = self.rotation[1].sin();

        let inv_scale = na::vector!(1.0 / self.scale[0], 1.0 / self.scale[1], 1.0 / self.scale[2]);

        na::matrix!(inv_scale[0] * (c1 * c3 + s1 * s2 * s3), inv_scale[1] * (c3 * s1 * s2 - c1 * s3), inv_scale[2] * (c2 * s1), 0.0;
                    inv_scale[0] * (c2 * s3)               , inv_scale[1] * (c2 * c3)               , inv_scale[2] * (-s2)    , 0.0;
                    inv_scale[0] * (c1 * s2 * s3 - c3 * s1), inv_scale[1] * (c1 * c3 * s2 + s1 * s3), inv_scale[2] * (c1 * c2), 0.0;
                    0.0                                    , 0.0                                    , 0.0                     , 0.0
        )
    }
}

pub struct PointLightComponent {
    pub light_intensity: f32
}

pub struct LveGameObject {
    pub model: Option<Rc<LveModel>>,
    pub color: na::Vector3<f32>,
    pub transform: TransformComponent,
    pub point_light: Option<PointLightComponent>
}

impl LveGameObject {
    pub fn new(
        model: Option<Rc<LveModel>>,
        color: Option<na::Vector3<f32>>,
        transform: Option<TransformComponent>,
    ) -> Self {

        let color = match color {
            Some(c) => c,
            None => na::vector![0.0, 0.0, 0.0],
        };

        let transform = match transform {
            Some(t) => t,
            None => TransformComponent {
                translation: na::vector![0.0, 0.0, 0.0],
                scale: na::vector![1.0, 1.0, 1.0],
                rotation: na::vector![0.0, 0.0, 0.0],
            }
        };

        Self {
            model,
            color,
            transform,
            point_light: None
        }
    }

    pub fn make_point_light(intensity: f32, radius: f32, color: na::Vector3<f32>) -> Self {
        let mut game_object = Self::new(
            None,
            Some(color),
            Some(TransformComponent {
                translation: na::vector![0.0, 0.0, 0.0],
                scale: na::vector![radius, 0.0, 0.0],
                rotation: na::vector![0.0, 0.0, 0.0],
            }));

        game_object.point_light = Some(PointLightComponent {
            light_intensity: intensity,
        });

        game_object
    }
}
