/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: obj/primitive/mod.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      An aggregator of primitive geometric items.
*/

pub use self::sphere::Sphere;
pub use self::vertex::{ Vertex_P, Vertex_PC, Vertex_PN, Vertex_PCN };

mod sphere;
mod vertex;

