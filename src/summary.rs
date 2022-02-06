use std::ops::AddAssign;

use async_std::task::JoinHandle;

use crate::file_ops::{COPY, MODIFY};

#[derive(Debug, Default)]
pub(crate) struct Summary {
    pub new: usize,
    pub existing: usize,
    pub updated: usize,
    pub errors: usize,
    pub total: usize,
}

impl Summary {
    pub(crate) async fn summarize(
        mut self,
        handles: Vec<JoinHandle<(&'static str, bool)>>,
    ) -> Self {
        for handle in handles {
            let (id, status) = handle.await;
            if !status {
                self.errors += 1;
            } else {
                match id {
                    COPY => {
                        self.new += 1;
                    }
                    MODIFY => {
                        self.updated += 1;
                    }
                    _ => unreachable!(),
                }
            }
        }
        self
    }
}

impl AddAssign for Summary {
    fn add_assign(&mut self, rhs: Self) {
        self.new += rhs.new;
        self.errors += rhs.errors;
        self.updated += rhs.updated;
        self.existing += rhs.existing;
        self.total += rhs.total;
    }
}
