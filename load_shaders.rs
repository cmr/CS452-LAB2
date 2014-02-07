#[crate_type = "rlib"];
#[crate_type = "lib"];

#[crate_id = "github.com/cmr/hgl-rs#hgl:0.0.1"];

#[license = "ASL2/MIT"];
#[comment = "Helper utilities for working with OpenGL"];

//! hgl-rs - helpers for working with OpenGL.
//!
//! hgl assumes GL 3.1 core profile with GLSL 140. It attempts to do complete
//! error checking, and return the information the GL exposes.
//!
//! *NOTE*: The various `activate` methods will explicitly bind the object,
//! but the other methods frequently bind themselves too! Be careful what you
//! call if you expect something to be bound to stay bound. They do not
//! restore the current binding before they return.

extern mod gl;

use gl::types::{GLint, GLuint, GLenum, GLsizei, GLchar, GLsizeiptr};
use std::libc::c_void;

/// Shader types
pub enum ShaderType {
    VertexShader,
    FragmentShader,
}

impl ShaderType {
    /// Convert a ShaderType into its corresponding GL value
    fn to_glenum(&self) -> GLenum {
        match *self {
            VertexShader => gl::VERTEX_SHADER,
            FragmentShader => gl::FRAGMENT_SHADER,
        }
    }
}

pub struct Shader {
    priv name: GLuint,
    priv type_: ShaderType
}

/// Get the complete info log, using the given function `info` and the given
/// enum value `status`, with the query function `get`. This is used to avoid
/// duplication for Program and Shader info logs.
///
/// It's easiest to just look at the usages below.
fn get_info_log(shader: GLuint, get: unsafe fn(GLuint, GLenum, *mut GLint),
                info: unsafe fn(GLuint, GLsizei, *mut GLint, *mut GLchar),
                status: GLenum) -> Option<~[u8]> {
    let mut ret = gl::FALSE as GLint;
    unsafe {
        get(shader, status, &mut ret);
    }

    if ret == gl::TRUE as GLint {
        return None
    }

    let mut len = 0;
    unsafe {
        get(shader, gl::INFO_LOG_LENGTH, &mut len as *mut GLint);
    }
    if len == 0 {
        return Some(~[]);
    }

    // len including trailing null
    let mut s = std::vec::with_capacity(len as uint - 1);

    unsafe {
        info(shader, len, &mut len as *mut GLsizei, s.as_mut_ptr() as *mut GLchar);
        s.set_len(len as uint - 1);
    }
    Some(s)
}

impl Shader {
    /// Create a shader from an id, making sure that it's actually a shader.
    pub fn from_name(name: GLuint, type_: ShaderType) -> Shader {
        if cfg!(not(ndebug)) {
            if gl::IsShader(name) == gl::FALSE {
                fail!("name is not a shader!");
            }
        }
        Shader::new_raw(name, type_)
    }

    /// Create a shader object without checking that the id is actually a
    /// shader
    fn new_raw(id: GLuint, type_: ShaderType) -> Shader {
        Shader { name: id, type_: type_ }
    }

    /// Returns the name (id) of the shader.
    pub fn name(&self) -> GLuint {
        self.name
    }

    /// Compile a shader.
    ///
    /// Takes the shader contents as a string. On success the Shader is returned.
    /// On failure, the complete log from glGetShaderInfoLog is returned.
    pub fn compile(source: &str, type_: ShaderType) -> Result<Shader, ~str> {
        let gltype = type_.to_glenum();
        let shader = gl::CreateShader(gltype);

        unsafe {
            gl::ShaderSource(shader, 1 as GLsizei, &(source.as_ptr() as *GLchar) as **GLchar,
                             &(source.len() as GLint) as *GLint);
        }
        gl::CompileShader(shader);

        match get_info_log(shader, gl::GetShaderiv, gl::GetShaderInfoLog, gl::COMPILE_STATUS) {
            Some(s) => Err(std::str::from_utf8_owned(s).expect("non-utf8 infolog!")),
            None    => Ok(Shader::new_raw(shader, type_))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl::DeleteShader(self.name);
    }
}

/// A program, which consists of multiple compiled shaders "linked" together
pub struct Program {
    name: GLuint
}

impl Program {
    /// Link shaders into a program
    pub fn link(shaders: &[Shader]) -> Result<Program, ~str> {
        let program = gl::CreateProgram();
        for shader in shaders.iter() {
            // there are no relevant errors to handle here.
            gl::AttachShader(program, shader.name);
        }
        gl::LinkProgram(program);

        match get_info_log(program, gl::GetProgramiv, gl::GetProgramInfoLog, gl::LINK_STATUS) {
            Some(s) => Err(std::str::from_utf8_owned(s).expect("non-utf8 infolog!")),
            None    => Ok(Program { name: program })
        }
    }

    pub fn activate(&self) {
        gl::UseProgram(self.name);
    }

    pub fn bind_frag(&self, color_number: GLuint, name: &str) {
        name.with_c_str(|cstr| unsafe {
            gl::BindFragDataLocation(self.name, color_number, cstr)
        });
    }

    pub fn uniform(&self, name: &str) -> GLint {
        name.with_c_str(|cstr| unsafe {
            gl::GetUniformLocation(self.name, cstr)
        })
    }

}
