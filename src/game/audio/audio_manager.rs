use super::Sound;

pub struct AudioManager {
    manager: kira::AudioManager,
}

impl AudioManager {
    pub fn new() -> AudioManager {
        let manager = kira::AudioManager::<kira::DefaultBackend>::
         new(kira::AudioManagerSettings::default())
         .expect("Failed to create an audio manager");

        AudioManager { manager }
    }

    pub fn play(&mut self, sound: &mut Sound) {
        self.manager.play(sound.get_data().clone())
         .expect("Failed to play sound");
    }
}