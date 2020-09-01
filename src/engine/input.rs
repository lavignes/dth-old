use crate::math::Vector2;

pub struct Input {
    stick0: Vector2,
    button0: bool,
    button1: bool,
    button2: bool,
    button3: bool,
    button4: bool,
    button5: bool,
}

impl Input {
    #[inline]
    pub fn stick0(&self) -> Vector2 {
        self.stick0
    }

    #[inline]
    pub fn set_stick0(&mut self, value: Vector2) {
        self.stick0 = value;
    }

    #[inline]
    pub fn button0(&self) -> bool {
        self.button0
    }

    #[inline]
    pub fn set_button0(&mut self, value: bool) {
        self.button0 = value;
    }

    #[inline]
    pub fn button1(&self) -> bool {
        self.button1
    }

    #[inline]
    pub fn set_button1(&mut self, value: bool) {
        self.button1 = value;
    }

    #[inline]
    pub fn button2(&self) -> bool {
        self.button2
    }

    #[inline]
    pub fn set_button2(&mut self, value: bool) {
        self.button2 = value;
    }

    #[inline]
    pub fn button3(&self) -> bool {
        self.button3
    }

    #[inline]
    pub fn set_button3(&mut self, value: bool) {
        self.button3 = value;
    }

    #[inline]
    pub fn button4(&self) -> bool {
        self.button4
    }

    #[inline]
    pub fn set_button4(&mut self, value: bool) {
        self.button4 = value;
    }

    #[inline]
    pub fn button5(&self) -> bool {
        self.button5
    }

    #[inline]
    pub fn set_button5(&mut self, value: bool) {
        self.button5 = value;
    }
}
