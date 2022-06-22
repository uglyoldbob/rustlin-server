enum MouseState {
    Normal,
    LeftButtonDown,
    LeftDragging,
    MiddleButtonDown,
    MiddleDragging,
    RightButtonDown,
    RightDragging,
}

pub enum MouseEventInput {
    /// The mouse moved to a particular position on the screen
    Move(i16, i16),
    LeftDown,
    LeftUp,
    MiddleDown,
    MiddleUp,
    RightDown,
    RightUp,
    ExtraDown,
    Extra2Down,
    /// The mouse wheel scrolled by a given amount
    Scrolling(i16),
}

pub enum MouseEventOutput {
    LeftDrag { from: (i16, i16), to: (i16, i16) },
    MiddleDrag { from: (i16, i16), to: (i16, i16) },
    RightDrag { from: (i16, i16), to: (i16, i16) },
    LeftClick((i16, i16)),
    MiddleClick((i16, i16)),
    RightClick((i16, i16)),
    ExtraClick,
    Extra2Click,
    Move((i16, i16)),
    Scrolling(i16),
}

pub struct Mouse {
    state: MouseState,
    events: Vec<MouseEventOutput>,
    start: (i16, i16),
    position: (i16, i16),
    scroll_amount: i16,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            state: MouseState::Normal,
            events: Vec::new(),
            start: (0, 0),
            position: (0, 0),
            scroll_amount: 0,
        }
    }

    pub fn events(&self) -> &Vec<MouseEventOutput> {
        &self.events
    }

    /// Signifies the end of events for a designated period of time, such as a frame of a game.
    pub fn parse(&mut self) {
        if self.scroll_amount != 0 {
            self.events
                .push(MouseEventOutput::Scrolling(self.scroll_amount));
            self.scroll_amount = 0;
        }
        if self.start != self.position {
            match self.state {
                MouseState::Normal => {
                    self.events.push(MouseEventOutput::Move(self.position));
                }
                MouseState::LeftButtonDown => {}
                MouseState::LeftDragging => {}
                MouseState::MiddleButtonDown => {}
                MouseState::MiddleDragging => {}
                MouseState::RightButtonDown => {}
                MouseState::RightDragging => {}
            }
        }
    }

    /// Indicates that all events so far have been processed
    pub fn clear(&mut self) {
        self.events.clear();
        self.start = self.position;
        self.scroll_amount = 0;
    }

    /// Parse on of the events for a period of time, to combined events where possible.
    pub fn event(&mut self, e: MouseEventInput) {
        match e {
            MouseEventInput::Move(x, y) => {
                self.position = (x, y);
            }
            MouseEventInput::LeftDown => {
                self.events.push(MouseEventOutput::LeftClick(self.position));
                match self.state {
                    MouseState::Normal => {
                        self.state = MouseState::LeftButtonDown;
                    }
                    MouseState::LeftButtonDown => {}
                    MouseState::LeftDragging => {}
                    MouseState::MiddleButtonDown => {}
                    MouseState::MiddleDragging => {}
                    MouseState::RightButtonDown => {}
                    MouseState::RightDragging => {}
                }
            }
            MouseEventInput::LeftUp => match self.state {
                MouseState::Normal => {}
                MouseState::LeftButtonDown => {
                    self.state = MouseState::Normal;
                }
                MouseState::LeftDragging => {
                    self.state = MouseState::Normal;
                    if self.start != self.position {
                        self.events.push(MouseEventOutput::LeftDrag {
                            from: self.start,
                            to: self.position,
                        });
                        self.start = self.position;
                    }
                }
                MouseState::MiddleButtonDown => {}
                MouseState::MiddleDragging => {}
                MouseState::RightButtonDown => {}
                MouseState::RightDragging => {}
            },
            MouseEventInput::MiddleDown => {
                self.events
                    .push(MouseEventOutput::MiddleClick(self.position));
                match self.state {
                    MouseState::Normal => {
                        self.state = MouseState::MiddleButtonDown;
                    }
                    MouseState::LeftButtonDown => {}
                    MouseState::LeftDragging => {}
                    MouseState::MiddleButtonDown => {}
                    MouseState::MiddleDragging => {}
                    MouseState::RightButtonDown => {}
                    MouseState::RightDragging => {}
                }
            }
            MouseEventInput::MiddleUp => match self.state {
                MouseState::Normal => {}
                MouseState::LeftButtonDown => {}
                MouseState::LeftDragging => {}
                MouseState::MiddleButtonDown => {
                    self.state = MouseState::Normal;
                }
                MouseState::MiddleDragging => {
                    self.state = MouseState::Normal;
                    if self.start != self.position {
                        self.events.push(MouseEventOutput::MiddleDrag {
                            from: self.start,
                            to: self.position,
                        });
                        self.start = self.position;
                    }
                }
                MouseState::RightButtonDown => {}
                MouseState::RightDragging => {}
            },
            MouseEventInput::RightDown => {
                self.events
                    .push(MouseEventOutput::RightClick(self.position));
                match self.state {
                    MouseState::Normal => {
                        self.state = MouseState::RightButtonDown;
                    }
                    MouseState::LeftButtonDown => {}
                    MouseState::LeftDragging => {}
                    MouseState::MiddleButtonDown => {}
                    MouseState::MiddleDragging => {}
                    MouseState::RightButtonDown => {}
                    MouseState::RightDragging => {}
                }
            }
            MouseEventInput::RightUp => match self.state {
                MouseState::Normal => {}
                MouseState::LeftButtonDown => {}
                MouseState::LeftDragging => {}
                MouseState::MiddleButtonDown => {}
                MouseState::MiddleDragging => {}
                MouseState::RightButtonDown => {
                    self.state = MouseState::Normal;
                }
                MouseState::RightDragging => {
                    self.state = MouseState::Normal;
                    if self.start != self.position {
                        self.events.push(MouseEventOutput::RightDrag {
                            from: self.start,
                            to: self.position,
                        });
                        self.start = self.position;
                    }
                }
            },
            MouseEventInput::ExtraDown => {
                self.events.push(MouseEventOutput::ExtraClick);
            }
            MouseEventInput::Extra2Down => {
                self.events.push(MouseEventOutput::Extra2Click);
            }
            MouseEventInput::Scrolling(amount) => {
                self.scroll_amount += amount;
            }
        }
    }
}
