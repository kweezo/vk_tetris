use kira::sound::static_sound::StaticSoundData;

pub struct Sound {
    data: StaticSoundData
}

impl Sound {
    pub fn new(path: &str, volume: f32, repeat: bool) -> Sound {
        let mut data = StaticSoundData::from_file(path)
         .expect(format!("Failed to parse {path}").as_str())
         .volume(volume);

        if repeat {
            data = data.loop_region(0.0f64..(data.num_frames() as f64));
        }

        Sound { data }
    }

    pub fn get_data(&mut self) -> &StaticSoundData {
        &self.data
    }
}