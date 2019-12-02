const SIZE: usize = 256;

#[derive(Default, Copy, Clone)]
pub struct Array<S> where 
    S:Default + Copy + Clone + PartialEq
{
    cnt: usize,
    conns: [S; SIZE],
}

impl <I> Array<I> where 
    I:Default + Copy + Clone + PartialEq
{
    pub fn has(&self, conn: &I) -> Result<usize, ()> {
        for i in 0..self.cnt {
            if self.conns[i] == *conn {
                return Ok(i);
            }
        }
        return Err(());
    }
    
    pub fn add(&mut self, conn: &I) {
        if self.cnt < SIZE {
            if let Err(_) = self.has(conn) {
                self.conns[self.cnt] = *conn;
                self.cnt += 1;
            }
        }
    }

    pub fn rmv_idx(&mut self, idx: usize) {
        if self.cnt > 0 {
            if idx < self.cnt {
                for j in idx..self.cnt - 1 {
                    self.conns[j] = self.conns[j + 1];
                }
                self.cnt -= 1;
            }
        }
    }

    pub fn rmv(&mut self, conn: &I) {
        if self.cnt > 0 {
            if let Ok(i) = self.has(conn) {
                for j in i..self.cnt - 1 {
                    self.conns[j] = self.conns[j + 1];
                }
                self.cnt -= 1;
            }
        }
    }
    
    pub fn iter_mut(&mut self) -> Take<IterMut<'_, I>> {
        self.conns.iter_mut().take(self.cnt)
    }
}
 
