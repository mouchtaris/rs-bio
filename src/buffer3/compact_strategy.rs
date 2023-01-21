use super::*;

impl<T: Copy> CompactStrategy<T> for SCopy {
    fn compact_within(slice: &mut [T], area: Range<usize>) {
        slice.copy_within(area, 0);
    }
}

impl<T> CompactStrategy<T> for SNone {
    fn compact_within(slice: &mut [T], Range { start, end }: Range<usize>) {
        if start == 0 || end == start {
            // done
            return;
        }
        //      end <= len
        //      1   <= start
        //      ------------ (+)
        //  =>  end + 1     <= len + start
        //   .  end - start <= len - 1
        //
        //      i     <= end - start - 1
        //  =>  i     <= len - 1 - 1
        //   .  i + 1 <= len - 1
        //
        //      i         <= end - start - 1
        //  =>  i + start <= end - 1
        //   .  i + start <= len - 1
        //
        // So all occurrences of (i + 1) and (start + i) are within bounds.
        for i in 0..(end - start) {
            let (dest, src) = slice.split_at_mut(i + 1);
            // When we split at (i + 1)
            // - dest[i]         == slice[i]
            // - src[0]          == slice[i + 1]
            // - src[start]      == slice[start + i + 1]
            // - src[start - 1]  == slice[start + i]
            // So this is:
            // swap(slice[i], slice[start + i])
            std::mem::swap(&mut dest[i], &mut src[start - 1]);
        }
    }
}
