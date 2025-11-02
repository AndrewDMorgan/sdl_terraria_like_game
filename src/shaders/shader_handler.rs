use metal::{Buffer, CommandQueue, CompileOptions, ComputePipelineDescriptor, ComputePipelineState, Device, MTLResourceOptions, MTLSize};

// this could be aligned with the alignment derive, but it's not needed
// as it's already aligned by the hard coded types
// 128 bits or 16 bytes
// the metal float4 type has no padding between elements either
// padding at the end doesn't really matter here (only when defining buffer sizes)
pub struct Float4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Float4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Float4 { x, y, z, w }
    }
}

/// Dictates which shader should be called in which situation
/// This should link the menue states to the shaders much easier
/// Each different window context should ideally have its own shader to reduce complexity
#[repr(u8)]
pub enum ShaderContext {
    GameLoop = 0,
    //MainMenu = 1,
    // etc...

    // the number of contexts available
    NumContexts = 1,
}

/// Handles a set of mutable shaders for different contexts
pub struct ShaderHandler {
    pub device: Device,
    shaders: [Shader; ShaderContext::NumContexts as usize],
}

impl From<ShaderError> for String {
    fn from(details: ShaderError) -> String {
        format!("[Shader Error] {:?}", details.details)
    }
}

impl ShaderHandler {
    pub fn new(device: Device, shaders: [Shader; ShaderContext::NumContexts as usize]) -> Self {
        ShaderHandler {
            device,
            shaders,
        }
    }

    pub fn execute(&self, context: ShaderContext, grid_size: MTLSize, threadgroup_size: MTLSize) {
        self.shaders[context as usize].execute(grid_size, threadgroup_size);
    }

    pub fn get_shader(&mut self, context: ShaderContext) -> &mut Shader {
        &mut self.shaders[context as usize]
    }
}

/// Handles a single shader, its pipeline state, and its buffers
pub struct Shader {
    pipeline_state: ComputePipelineState,
    command_queue: CommandQueue,
    buffers: Vec<Buffer>,
}

impl Shader {
    /// Creates a new shader from the given device, source file, buffer sizes, and entry function name
    pub fn new(device: &Device, source: &str, buffer_sizes: &[u64], entry_function_name: &str) -> Result<Self, ShaderError> {
        let src = std::fs::read_to_string(source)
            .map_err(|e| ShaderError { details: format!("Failed to read shader file: {}", e) })?;
        // compiling the shaders and getting them ready
        let opts = CompileOptions::new();
        let lib = device.new_library_with_source(&src, &opts)
            .map_err(|e| ShaderError { details: format!("Failed to compile Metal shader: {}", e) })?;
        
        let func = lib.get_function(entry_function_name, None)
            .map_err(|e| ShaderError {
                details: format!("Failed to locate and get function '{}' from Metal library (Please verify the name is correct): {}", entry_function_name, e)
        })?;
        let desc = ComputePipelineDescriptor::new();
        desc.set_compute_function(Some(&func));
        
        let pipeline_state = device
            .new_compute_pipeline_state(&desc)
            .map_err(|e| ShaderError { details: format!("Failed to create compute pipeline state: {}", e) })?;
        
        let command_queue = device.new_command_queue();

        let mut buffers = Vec::new();
        for buffer_size in buffer_sizes {
            buffers.push(
                // there can't really be any data in this yet as this function doesn't request the user to provide that
                device.new_buffer(
                    *buffer_size,
                    MTLResourceOptions::StorageModeShared
                )
            );
        }

        Ok(Shader {
            pipeline_state,
            command_queue,
            buffers,
        })
    }

    /// Updates the data in the specified buffer
    pub fn update_buffer<T>(&mut self, index: usize, data: T) -> Result<(), ShaderError> {
        let ptr = self.buffers[index].contents() as *mut T;
        if ptr.is_null() {
            return Err(ShaderError { details: "Failed to get buffer contents; the pointer to its contents was null.".to_string() });
        }
        unsafe { *ptr = data; }
        Ok(())
    }

    /// Updates the data in the specified buffer from a slice
    pub fn update_buffer_slice<T>(&mut self, index: usize, data: &[T]) -> Result<(), ShaderError> {
        let ptr = self.buffers[index].contents() as *mut T;
        if ptr.is_null() {
            return Err(ShaderError { details: "Failed to get buffer contents; the pointer to its contents was null.".to_string() });
        }
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        }
        Ok(())
    }

    /// Executes the shader with the given grid and threadgroup sizes
    pub fn execute(&self, grid_size: MTLSize, threadgroup_size: MTLSize) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline_state);

        // attaching the buffers
        for (i, buffer) in self.buffers.iter().enumerate() {
            encoder.set_buffer(i as u64, Some(buffer), 0);
        }

        encoder.dispatch_threads(grid_size, threadgroup_size);
        
        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
    }

    /// Executes the shader with the given grid and threadgroup sizes, plus extra buffers
    /// This should rarely be used, only in cases where buffer sizes may vary between uses
    /// Creating new buffers incurs an extra performance cost which in most situations is unnecessary
    pub fn execute_with_extra_buffers<'a>(
        &self,
        extra_buffers: &[&Buffer],
        grid_size: MTLSize,
        threadgroup_size: MTLSize,
        callback: Option<Box<dyn FnOnce() + 'a>>,
    ) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline_state);

        // attaching the buffers
        for (i, buffer) in self.buffers.iter().enumerate() {
            encoder.set_buffer(i as u64, Some(buffer), 0);
        }
        for (i, buffer) in extra_buffers.iter().enumerate() {
            encoder.set_buffer((self.buffers.len() + i) as u64, Some(buffer), 0);
        }

        encoder.dispatch_threads(grid_size, threadgroup_size);
        encoder.end_encoding();
        command_buffer.commit();

        // do any extra work here (the gpu is running, but not yet joined)
        if let Some(callback) = callback { callback(); }

        command_buffer.wait_until_completed();
    }

    /// Gets a mutable pointer to the contents of the specified buffer
    pub fn get_buffer_contents<T>(&self, index: usize) -> *mut T {
        self.buffers[index].contents() as *mut T
    }
}

/// Represents an error that occurred while handling shaders
#[derive(Debug)]
pub struct ShaderError {
    details: String,
}

