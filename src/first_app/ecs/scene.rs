use super::entity::*;

use std::str::FromStr;

pub struct Scene {
    name: String,
    entities: Vec<Entity>,
    entity_id: usize
}

impl Scene {
    pub fn new_from_file(path: &str) {
        todo!();
    }

    pub fn new(name: &str, entities: &Vec<Entity>) {

    }

    pub fn new_null(name: &str) -> Self {
        Self { 
            name: String::from_str(name).unwrap(),
            entities: Vec::new(),
            entity_id: 0
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn update(&mut self, frame_time: f32) {
        todo!();
    }

    pub fn render(&self) {
        for entity in self.entities.iter() {
            match &entity.model {
                Some(_) => { entity.model.as_ref().unwrap().render() },
                None => {}
            }
        }
    }

    pub fn display_info(&mut self, ui: &imgui::Ui) {
        imgui::Window::new("Scene").size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(&ui, || {
            let mut current_id: usize = 0;
            let mut bruh = false;
            for entity in self.entities.iter_mut() {
                current_id += 1;
                if ui.button(&entity.name) {
                    entity.selected = true;
                    self.entity_id = current_id -1;
                    bruh = true;
                }
            }
            if bruh {
                self.entities[self.entity_id].selected = false;
            }

        });

        imgui::Window::new("Entity").size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(&ui, || {
            self.entities[self.entity_id].display_info(ui);
        });
    }
}