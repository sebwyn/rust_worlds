use cgmath::Point3;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Debug)]
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    fn scale(self, scale: f32) -> Vec2 {
        Vec2 { x: self.x * scale, y: self.y * scale }
    }

    fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2f32) + self.y.powf(2f32))
    }

    //slow rotation in polar space
    pub fn rotate(&self, angle: f32) -> Vec2 {
        let magnitude = self.magnitude();
        let current_angle = if self.y > 0f32 {
             f32::acos(self.x / magnitude)
        } else {
            2f32 * std::f32::consts::PI - f32::acos(self.x / magnitude)
        };
        let new_angle = current_angle + angle; 
        let (new_y, new_x) = f32::sin_cos(new_angle);
        Vec2 { x: magnitude * new_x, y: magnitude * new_y }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Debug)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl Vec3 {
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2f32) + self.y.powf(2f32) + self.z.powf(2f32))
    }

    pub fn normalize(&self) -> Self {
        let magnitude = self.magnitude();
        Self { x: self.x / magnitude, y: self.y / magnitude, z: self.z / magnitude }
    }

    /*fn scale(self, scale: f32) -> Vec2 {
        Vec2 { x: self.x * scale, y: self.y * scale }
    }*/


    //slow rotation in polar space
    /*pub fn rotate(&self, angle: f32) -> Vec2 {
        let magnitude = self.magnitude();
        let current_angle = if self.y > 0f32 {
             f32::acos(self.x / magnitude)
        } else {
            2f32 * std::f32::consts::PI - f32::acos(self.x / magnitude)
        };
        let new_angle = current_angle + angle; 
        let (new_y, new_x) = f32::sin_cos(new_angle);
        Vec2 { x: magnitude * new_x, y: magnitude * new_y }
    }*/
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Into<Point3<f32>> for Vec3 {
    fn into(self) -> Point3<f32> {
        Point3 { x: self.x, y: self.y, z: self.z }
    }
}

impl From<cgmath::Vector4<f32>> for Vec3 {
    fn from(vec4: cgmath::Vector4<f32>) -> Vec3 {
        Vec3 { x: vec4.x, y: vec4.y, z: vec4.z }
    }
}

impl Into<cgmath::Vector3<f32>> for Vec3 {
    fn into(self) -> cgmath::Vector3<f32> {
        cgmath::Vector3 { x: self.x, y: self.y, z: self.z }
    }
}
