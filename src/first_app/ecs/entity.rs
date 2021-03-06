use std::str::FromStr;

use super::model::*;
//use Model as ModelComponent;

use nalgebra as na;

use std::rc::Rc;

pub struct NewTransformComponent {
    pub translation: na::Vector3<f32>,
    pub scale: na::Vector3<f32>,
    pub rotation: na::Vector3<f32>,
}

impl NewTransformComponent {
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
    color: na::Vector3<f32>,
    intensity: f32,
    radius: f32
}

pub struct SpotLightComponent {
    color: na::Vector3<f32>,
    intensity: f32,
    direction: na::Vector3<f32>,
    cut_off: f32,
    outer_cut_off: f32,
    radius: f32
}

pub struct DirectionalLightComponent {
    color: na::Vector3<f32>,
    intensity: f32,
    direction: na::Vector3<f32>,
}

pub struct Entity {
    pub name: String,
    pub selected: bool,
    pub transform: NewTransformComponent,
    pub model: Option<Rc<Model>>,
    pub point_light: Option<PointLightComponent>,
    pub spot_light: Option<SpotLightComponent>,
    pub directional_light: Option<DirectionalLightComponent>,
}

impl Entity {
    pub fn new(name: &str, transform: NewTransformComponent) -> Self {
        Self {
            name: String::from_str(name).unwrap(),
            selected: false,
            transform,
            model: None,
            point_light: None,
            spot_light: None,
            directional_light: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, component: Rc<Model>) {
        self.model = Some(component);
    }

    #[allow(dead_code)]
    pub fn set_point_light(&mut self, component: PointLightComponent) {
        self.point_light = Some(component);
        self.spot_light = None;
        self.directional_light = None;
    }

    #[allow(dead_code)]
    pub fn set_spot_light(&mut self, component: SpotLightComponent) {
        self.point_light = None;
        self.spot_light = Some(component);
        self.directional_light = None;
    }

    #[allow(dead_code)]
    pub fn set_directional_light(&mut self, component: DirectionalLightComponent) {
        self.point_light = None;
        self.spot_light = None;
        self.directional_light = Some(component);
    }

    #[allow(dead_code)]
    pub fn add_point_light(&mut self) {
        self.point_light = Some(PointLightComponent {
            color: na::vector![1.0, 1.0, 1.0],
            intensity: 0.0,
            radius: 0.0
        });
        self.spot_light = None;
        self.directional_light = None;
    }

    #[allow(dead_code)]
    pub fn add_spot_light(&mut self) {
        self.spot_light = Some(SpotLightComponent {
            color: na::vector![1.0, 1.0, 1.0],
            intensity: 0.0,
            direction: na::vector![0.0, 0.0, 0.0],
            cut_off: 0.0,
            outer_cut_off: 0.0,
            radius: 0.0
        });
        self.spot_light = None;
        self.directional_light = None;
    }

    #[allow(dead_code)]
    pub fn add_directional_light(&mut self) {
        self.directional_light = Some(DirectionalLightComponent {
            color: na::vector![0.0, 0.0, 0.0],
            intensity: 0.0,
            direction: na::vector![0.0, 0.0, 0.0]
        });
        self.spot_light = None;
        self.point_light = None;
    }

    pub fn display_info(&mut self, ui: &imgui::Ui) {
        ui.input_text("Name", &mut self.name).build();
        ui.separator();

        {
            let pre_transfrom = self.transform.translation;
            let mut transfrom = [pre_transfrom.x, pre_transfrom.y, pre_transfrom.z];
            ui.input_float3("Translation", &mut transfrom).build();
            self.transform.translation.x = transfrom[0];
            self.transform.translation.y = transfrom[1];
            self.transform.translation.z = transfrom[2];

            let pre_rotation = self.transform.rotation;
            let mut rotation = [pre_rotation.x, pre_rotation.y, pre_rotation.z];
            ui.input_float3("Rotation", &mut rotation).build();
            self.transform.rotation.x = rotation[0];
            self.transform.rotation.y = rotation[1];
            self.transform.rotation.z = rotation[2];

            let pre_scale = self.transform.scale;
            let mut scale = [pre_scale.x, pre_scale.y, pre_scale.z];
            ui.input_float3("Scale", &mut scale).build();
            self.transform.scale.x = scale[0];
            self.transform.scale.y = scale[1];
            self.transform.scale.z = scale[2];
        }

        match &self.point_light {
            Some(_) => {
                ui.separator();
                let light = self.point_light.as_ref().unwrap();
                let mut color = [light.color.x, light.color.y, light.color.z];
                let mut intensity = light.intensity;
                let mut radius = light.radius;
                ui.input_float3("Color", &mut color).build();
                ui.input_float("Intensity", &mut intensity).build();
                ui.input_float("Radius", &mut radius).build();
                self.point_light = Some(
                    PointLightComponent { 
                        color: na::vector![color[0], color[1], color[2]], 
                        intensity, 
                        radius
                    }
                );
            }
            None => {}
        }

        match &self.spot_light {
            Some(_) => {
                ui.separator();
                let light = self.spot_light.as_ref().unwrap();
                let mut color = [light.color.x, light.color.y, light.color.z];
                let mut intensity = light.intensity;
                let mut direction = [light.direction.x, light.direction.y, light.direction.z];
                let mut cut_off = light.cut_off;
                let mut outer_cut_off = light.outer_cut_off;
                let mut radius = light.radius;
                ui.input_float3("Color", &mut color).build();
                ui.input_float("Intensity", &mut intensity).build();
                ui.input_float3("Direction", &mut direction).build();
                ui.input_float("Cut off", &mut cut_off).build();
                ui.input_float("Outer cut off", &mut outer_cut_off).build();
                ui.input_float("Radius", &mut radius).build();
                self.spot_light = Some(
                    SpotLightComponent {
                        color: na::vector![color[0], color[1], color[2]],
                        intensity,
                        direction: na::vector![direction[0], direction[1], direction[2]],
                        cut_off,
                        outer_cut_off,
                        radius
                    }
                )

            }
            None => {}
        }

        match &self.directional_light {
            Some(_) => {
                ui.separator();
                let light = self.directional_light.as_ref().unwrap();
                let mut color = [light.color.x, light.color.y, light.color.z];
                let mut intensity = light.intensity;
                let mut direction = [light.direction.x, light.direction.y, light.direction.z];
                ui.input_float3("Color", &mut color).build();
                ui.input_float("Intensity", &mut intensity).build();
                ui.input_float3("Direction", &mut direction).build();
                self.directional_light = Some(
                    DirectionalLightComponent { 
                        color: na::vector![color[0], color[1], color[2]], 
                        intensity, direction: 
                        na::vector![direction[0], direction[1], direction[2]]
                    }
                );
            }
            None => {}
        }

    }

    /*pub fn render(&self) {
        self.model.as_ref().unwrap().render();
    }*/
}