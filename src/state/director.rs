/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: state/director.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      A state stack that propogates update
      and render signals.
*/

use std::local_data;

static tls_key: local_data::Key<@mut Director> = &local_data::Key;

#[allow(default_methods)]
pub trait State
{
  pub fn load(&mut self);
  pub fn unload(&mut self)
  { }

  /*  Returns true when the event has been captured. If the event is not
      captured, it's set to the next lower state. (A state can capture
      renders, for example, as an optimization -- or updates as a sanity or
      security measure). Rinse and repeat. */
  pub fn update(&mut self, _delta: f32) -> bool /* dt is in terms of seconds. */
  { false }
  pub fn render(&mut self) -> bool
  { false }

  /* TODO: Trait inheritance with Input_Listener. */
  pub fn key_action(&mut self, _key: i32, _action: i32, _mods: i32) -> bool
  { false }
  pub fn key_char(&mut self, _ch: char) -> bool
  { false }
  pub fn mouse_action(&mut self, _button: i32, _action: i32, _mods: i32) -> bool
  { false }
  pub fn mouse_moved(&mut self, _x: f32, _y: f32) -> bool
  { false }
}

pub struct Director
{
  states: ~[@mut State],
}

impl Director
{
  pub fn new() -> @mut Director
  {
    let director = @mut Director
    {
      states: ~[],
    };

    /* Store the director in task-local storage. (singleton) */
    local_data::set(tls_key, director);

    director
  }

  /* Accesses the singleton director from task-local storage. */
  pub fn get() -> @mut Director
  {
    local_data::get(tls_key, 
    |opt|
    {
      match opt
      {
        Some(x) => *x,
        None => fail!("Singleton not available")
      }
    })
  }

  pub fn push(&mut self, mut state: @mut State)
  {
    state.load();
    self.states.push(state);
  }

  pub fn pop(&mut self)
  {
    let mut state = self.states.pop();
    state.unload();
  }

  /** Updating and rendering. **/
  pub fn update(&mut self, delta: f32)
  {
    assert!(self.states.len() > 0);

    for self.states.mut_iter().advance |x|
    {
      if x.update(delta)
      { break; }
    }
  }

  pub fn render(&mut self)
  {
    assert!(self.states.len() > 0);

    for self.states.mut_iter().advance |x|
    {
      if x.render()
      { break; }
    }
  }

  /** Input handling. **/
  pub fn key_action(&mut self, key: i32, action: i32, mods: i32)
  {
    assert!(self.states.len() > 0);

    for self.states.mut_rev_iter().advance |x|
    {
      if x.key_action(key, action, mods)
      { break; }
    }
  }

  pub fn key_char(&mut self, ch: char)
  {
    assert!(self.states.len() > 0);

    for self.states.mut_rev_iter().advance |x|
    {
      if x.key_char(ch)
      { break; }
    }
  }

  pub fn mouse_action(&mut self, button: i32, action: i32, mods: i32)
  {
    assert!(self.states.len() > 0);

    for self.states.mut_rev_iter().advance |x|
    {
      if x.mouse_action(button, action, mods)
      { break; }
    }
  }

  pub fn mouse_moved(&mut self, x: f32, y: f32)
  {
    assert!(self.states.len() > 0);

    for self.states.mut_rev_iter().advance |state|
    {
      if state.mouse_moved(x, y)
      { break; }
    }
  }
}

