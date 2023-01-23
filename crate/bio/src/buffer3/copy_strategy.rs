use super::*;

impl<T: Copy> CopyStrategy<T> for SCopy {
    fn copy_slice(dest: &mut [T], src: &[T]) {
        dest.copy_from_slice(src);
    }
}

impl<T: Clone> CopyStrategy<T> for SClone {
    fn copy_slice(dest: &mut [T], src: &[T]) {
        for i in 0..src.len() {
            dest[i] = src[i].clone();
        }
    }
}
