use atomic_float::AtomicF32;
use nalgebra::{base::Vector3, UnitQuaternion};

pub struct AtomicQuaternion {
    a: AtomicF32,
    b: AtomicF32,
    c: AtomicF32,
    d: AtomicF32,
}

impl From<nalgebra::UnitQuaternion<f32>> for AtomicQuaternion {
    fn from(quat: nalgebra::UnitQuaternion<f32>) -> AtomicQuaternion {
        if let [a, b, c, d] = quat.as_vector().as_slice() {
            AtomicQuaternion{
                a: AtomicF32::new(*a),
                b: AtomicF32::new(*b),
                c: AtomicF32::new(*c),
                d: AtomicF32::new(*d),
            }
        } else {
            unreachable!();
        }
    }
}

impl AtomicQuaternion {
    fn load(&self) -> nalgebra::Quaternion<f32> {
        let a = self.a.load(Ordering::Relaxed);
        let b = self.a.load(Ordering::Relaxed);
        let c = self.a.load(Ordering::Relaxed);
        let d = self.a.load(Ordering::Relaxed);
        let vec = nalgebra::Vector4::from_column_slice(&[a, b, c, d]);
        nalgebra::Quaternion::from_vector(vec)
    }

    fn store(&self, quat: nalgebra::UnitQuaternion<f32>) {
        if let [a, b, c, d] = quat.as_vector().as_slice() {
            self.a.store(*a, Ordering::Relaxed);
            self.b.store(*b, Ordering::Relaxed);
            self.c.store(*c, Ordering::Relaxed);
            self.d.store(*d, Ordering::Relaxed);
        } else {
            unreachable!();
        }
    }

}
