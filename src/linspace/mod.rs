// TODO: this copied from crate itertools to implement Clone for Linspace.
// I should probably make a pull request, since there is no reason (afaik)
// for the Linspace struct not to be Clone

pub struct Linspace {
    start: f32,
    step: f32,
    index: usize,
    len: usize,
}

impl Clone for Linspace {
    fn clone(&self) -> Self {
        Linspace {
            start: self.start.clone(),
            step: self.step.clone(),
            index: self.index.clone(),
            len: self.len.clone(),
        }
    }
}

impl Iterator for Linspace {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if self.index >= self.len {
            None
        } else {
            // Calculate the value just like numpy.linspace does
            let i = self.index;
            self.index += 1;
            Some(self.start + self.step * (i as f32))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len - self.index;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for Linspace {
    #[inline]
    fn next_back(&mut self) -> Option<f32> {
        if self.index >= self.len {
            None
        } else {
            // Calculate the value just like numpy.linspace does
            self.len -= 1;
            let i = self.len;
            Some(self.start + self.step * (i as f32))
        }
    }
}

impl ExactSizeIterator for Linspace {}

/// Return an iterator of evenly spaced floats.
///
/// The `Linspace` has `n` elements, where the first
/// element is `a` and the last element is `b`.
///
/// Iterator element type is `F`, where `F` must be
/// either `f32` or `f64`.
///
/// ```
/// use itertools::linspace;
///
/// itertools::assert_equal(linspace::<f32>(0., 1., 5),
///                         vec![0., 0.25, 0.5, 0.75, 1.0]);
/// ```
#[inline]
pub fn linspace(a: f32, b: f32, n: usize) -> Linspace {
    let step = if n > 1 {
        let nf = n as f32;
        (b - a) / (nf - (1 as f32))
    } else {
        0 as f32
    };
    Linspace {
        start: a,
        step: step,
        index: 0,
        len: n,
    }
}
