/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: client/md5/model/model.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      Loads, parses, and represents
      the model (as in MVC) side of
      MD5 animated models.
*/

use std::{ io, path, vec, str };
use super::{ Joint, Vertex, Triangle, Weight, Mesh, Animation };
use math;
use log::Log;

#[macro_escape]
#[path = "../../../shared/log/macros.rs"]
mod macros;

struct Model
{
  file_directory: ~str,

  version: i32,
  num_joints: i32,
  num_meshes: i32,
  is_animated: bool,

  joints: ~[Joint],
  meshes: ~[Mesh],

  animation: Option<Animation>,
  
  local_to_world: math::Mat4x4,
}

impl Model
{
  /* TODO: Return Option. */
  pub fn new(mesh_file: ~str) -> Model
  {
    /* TODO: Custom Path type to handle this. */
    let dir;
    let mut posix = true;
    for x in range(0, mesh_file.len() as i32)
    { if mesh_file[x] == '\\' as u8 { posix = false; } }
    if posix
    { dir = path::PosixPath(mesh_file.clone()).normalize().dirname(); }
    else
    { dir = path::WindowsPath(mesh_file.clone()).normalize().dirname(); }

    let mut model = Model
    {
      file_directory: dir,
      version: 0,
      num_joints: 0,
      num_meshes: 0,
      is_animated: false,

      joints: ~[],
      meshes: ~[],

      animation: None,

      local_to_world: math::Mat4x4::new(),
    };

    model.load(mesh_file);

    model
  }

  fn load(&mut self, file: ~str) -> bool
  {
    let fior = io::file_reader(&path::Path(file));
    if fior.is_err()
    { log_error!("Failed to open model file %s", file); return false; }

    /* Clear existing data. */
    self.joints.clear();
    self.meshes.clear();

    let fio = fior.unwrap();
    let mut param;
    macro_rules! read_param
    (
      () =>
      ({
        param = ~"";
        let mut ch = fio.read_char();
        while ch.is_whitespace() && !fio.eof() /* Find the next word. */
        { ch = fio.read_char(); }

        if !fio.eof()
        { 
          param.push_char(ch);
          ch = fio.read_char();
          while !ch.is_whitespace() && !fio.eof() /* Read the next word. */
          { param.push_char(ch); ch = fio.read_char(); }
        }
      });
    )
    macro_rules! read_junk
    (() => ({ read_param!(); });)
    macro_rules! ignore_line
    (() => ({ fio.read_line(); });)
    macro_rules! read_type
    (
      ($var:expr) =>
      ({
        let name = param.clone();
        read_param!();
        let num = FromStr::from_str(param);
        if num.is_some()
        { $var = num.unwrap(); }
        else
        { log_error!("Invalid %s in %s", name, file); }
      });
    )

    /* Read the first param and jump into the parsing. */
    log_debug!("Parsing model %s", file);
    log_push!();
    read_param!();
    while !fio.eof()
    {
      match param
      {
        ~"MD5Version" =>
        {
          /* Read version. */
          read_type!(self.version);
          log_debug!("Model version: %d", self.version as int);
        }
        ~"commandline" =>
        { ignore_line!(); }
        ~"numJoints" =>
        {
          read_type!(self.num_joints);
          self.joints = vec::with_capacity(self.num_joints as uint);
          log_debug!("Model joints: %d", self.num_joints as int);
        }
        ~"numMeshes" =>
        {
          read_type!(self.num_meshes);
          self.meshes = vec::with_capacity(self.num_meshes as uint);
          log_debug!("Model meshes: %d", self.num_meshes as int);
        }
        ~"joints" =>
        {
          let mut joint = Joint::new();
          read_junk!();
          log_debug!("Reading model joints");
          log_push!();
          
          for _ in range(0, self.num_joints)
          {
            read_param!();
            joint.name = param.clone();

            read_type!(joint.parent);

            read_junk!();
            read_type!(joint.position.x);
            read_type!(joint.position.y);
            read_type!(joint.position.z);
            read_junk!();
            read_junk!();
            read_type!(joint.orientation.x);
            read_type!(joint.orientation.y);
            read_type!(joint.orientation.z);
            read_junk!();

            joint.orientation.compute_w();
            self.joints.push(joint.clone());

            /* Ignore the rest of the line. */
            ignore_line!();
          }
          log_pop!();

          read_junk!();
        }
        ~"mesh" =>
        {
          let mut mesh = Mesh::new();
          let mut vert = Vertex::new();
          let mut tri = Triangle::new();
          let mut weight = Weight::new();
          let mut num_verts = 0;
          let mut num_tris = 0;
          let mut num_weights = 0;

          log_debug!("Parsing mesh");
          log_push!();

          read_junk!();
          read_param!();
          while param != ~"}"
          {
            match param
            {
              ~"shader" => /* shader == texture path */
              {
                read_param!();

                /* Rust needs to know if it's a POSIX or Windows
                 * path so it can parse it. Just loop through and
                 * check for \ which indicates it's for Windows. */
                let mut posix = true;
                for x in range(0, param.len() as i32)
                { if param[x] == '\\' as u8 { posix = false; } }
                if posix
                {
                  let path = path::PosixPath(param.clone()).normalize();
                  mesh.texture = match path.filename()
                    {
                      Some(file) => { file.to_owned() },
                      None => { ~"" }
                    };
                }
                else
                {
                  let path =  path::WindowsPath(param.clone()).normalize();
                  mesh.texture = match path.filename()
                    {
                      Some(file) => { file.to_owned() },
                      None => { ~"" }
                    };
                }
                /* Remove quotes. */
                mesh.texture = self.file_directory + "/" + str::replace(mesh.texture, "\"", "");
                log_debug!("Mesh shader/texture: %s", mesh.texture);

                ignore_line!();
              }
              ~"numverts" =>
              {
                read_type!(num_verts);
                ignore_line!();

                log_debug!("Mesh verts: %d", num_verts);

                for _ in range(0, num_verts)
                {
                  read_junk!();
                  read_junk!();
                  read_junk!();
                  read_type!(vert.tex_coord.x);
                  read_type!(vert.tex_coord.y);
                  read_junk!();
                  read_type!(vert.start_weight);
                  read_type!(vert.weight_count);

                  ignore_line!();

                  mesh.verts.push(vert);
                  mesh.tex_coords.push(vert.tex_coord);
                }
              }
              ~"numtris" =>
              {
                read_type!(num_tris);
                ignore_line!();
                log_debug!("Mesh tris: %d", num_tris);

                for _ in range(0, num_tris)
                {
                  read_junk!();
                  read_junk!();
                  read_type!(tri.indices[0]);
                  read_type!(tri.indices[1]);
                  read_type!(tri.indices[2]);

                  ignore_line!();

                  mesh.triangles.push(tri);
                  mesh.indices.push(tri.indices[0] as u32);
                  mesh.indices.push(tri.indices[1] as u32);
                  mesh.indices.push(tri.indices[2] as u32);
                }
              }
              ~"numweights" =>
              {
                read_type!(num_weights);
                ignore_line!();
                log_debug!("Mesh weights: %d", num_weights);

                for _ in range(0, num_weights)
                {
                  read_junk!();
                  read_junk!();
                  read_type!(weight.joint_id);
                  read_type!(weight.bias);
                  read_junk!();
                  read_type!(weight.position.x);
                  read_type!(weight.position.y);
                  read_type!(weight.position.z);
                  read_junk!();

                  ignore_line!();
                  mesh.weights.push(weight);
                }
              }
              _ =>
              { ignore_line!(); }
            }

            read_param!();
          }

          self.prepare_mesh(&mut mesh);
          self.meshes.push(mesh);
          log_pop!();
        }
        _ => { } 
      }

      /* Move to the next param. */
      read_param!();
    }
    log_pop!();

    true
  }

  fn prepare_mesh(&mut self, mesh: &mut Mesh)
  {
    mesh.positions.clear();
    mesh.tex_coords.clear();

    for x in range(0, mesh.verts.len() as i32)
    {
      let vert = &mut mesh.verts[x];
      vert.position = math::Vec3f::zero();
      vert.normal = math::Vec3f::zero();

      /* Sum the position of all the weights. */
      for w in range(0, vert.weight_count)
      {
        let weight = &mut mesh.weights[vert.start_weight + w];
        let joint = &mut self.joints[weight.joint_id];

        /* Convert the weight position from joint local to object space. */
        let rot_pos = joint.orientation.rotate_vec(&weight.position);
        
        vert.position = vert.position + ((joint.position + rot_pos) * weight.bias);
      }

      mesh.positions.push(vert.position);
      mesh.tex_coords.push(vert.tex_coord);
    }
  }

  fn prepare_mesh_with_skeleton(&mut self, mesh_index: i32)
  {
    let skel = &self.animation.get_mut_ref().animated_skeleton;
    let mesh = &mut self.meshes[mesh_index];

    for i in range(0, mesh.verts.len())
    {
      let vert = &mesh.verts[i];
      let position = &mut mesh.positions[i];
      //let normal = &mut mesh.normals[i];

      *position = math::Vec3f::zero();
      //*normal = math::Vec3f::zero();

      for m in range(0, vert.weight_count)
      {
        let weight = &mesh.weights[vert.start_weight + m];
        let joint = &skel.joints[weight.joint_id];

        let rot_pos = joint.orientation.rotate_vec(&weight.position);
        *position = *position + ((joint.position + rot_pos) * weight.bias);
        //*normal = *normal + (joint.orientation.rotate_vec(&vert.normal) * weight.bias);
      }
    }
  }

  pub fn update(&mut self, dt: f32)
  {
    if self.animation.is_some()
    {
      self.animation.get_mut_ref().update(dt);

      for i in range(0, self.meshes.len())
      { self.prepare_mesh_with_skeleton(i as i32); }
    }
  }

  pub fn load_animation(&mut self, file: ~str) -> bool
  {
    self.animation = Animation::new(file);
    if self.animation.is_some()
    {
      let valid = self.check_animation(self.animation.get_ref());
      if !valid
      { self.animation = None; }
    }

    self.animation.is_some()
  }

  fn check_animation(&self, animation: &Animation) -> bool
  {
    if self.num_joints != animation.num_joints
    { return false; }

    for i in range(0, self.joints.len())
    {
      let mesh_joint = &self.joints[i];
      let anim_joint = &animation.joint_infos[i];

      if  mesh_joint.name != anim_joint.name ||
          mesh_joint.parent != anim_joint.parent_id
      { return false; }
    }

    true
  }
}

