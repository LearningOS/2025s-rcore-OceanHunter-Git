use core::cmp::max;

use alloc::vec;
use alloc::vec::Vec;

/// pub banker
pub struct Banker{
    /// available
    pub available: Vec<i32>,
    /// allocation
    pub allocation: Vec<Vec<i32>>,
    /// need
    pub need: Vec<Vec<i32>>,
}

impl Banker{
    pub fn new() -> Self{
        return Self{
            available: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new()
        };
    }

    pub fn add_new_resource(&mut self, rid:usize, cnt:usize){
        while self.available.len() <= rid{
            self.available.push(0);
        }
        self.available[rid] = cnt as i32;
    }

    fn expand(&mut self, tid:usize, rid:usize){
        let resource_count = max(rid + 1, self.available.len());

        for i in 0..self.allocation.len(){
            while self.allocation[i].len() < resource_count{
                self.allocation[i].push(0);
            }
        }

        for i in 0..self.need.len(){
            while self.need[i].len() < resource_count{
                self.need[i].push(0);
            }
        }

        while self.allocation.len() <= tid {
            self.allocation.push(vec![0;resource_count]);
        }

        while self.need.len() <= tid {
            self.need.push(vec![0;resource_count]);
        }

    }

    pub fn add_allocation(&mut self, tid:usize, rid:usize, delta:i32){
        self.expand(tid, rid);
        self.allocation[tid][rid] += delta;
    }

    pub fn add_need(&mut self, tid:usize, rid:usize, delta:i32){
        self.expand(tid, rid);
        self.need[tid][rid] += delta;
    }

    pub fn add_available(&mut self, rid:usize, delta:i32){
        self.available[rid] += delta;
    }

    pub fn check(&self) -> bool{
        let resource_count = self.available.len();
        let thread_count = self.allocation.len();
        let mut work:Vec<i32> = self.available.clone();
        let mut finish:Vec<bool> = vec![false;thread_count];
        let mut finish_count = 0;

        while finish_count < thread_count{
            let mut no_new_finish = true;
            for i in 0..thread_count {
                if finish[i] {
                    continue;
                }
                let mut flag = true;
                for j in 0..resource_count{
                    if work[j] < self.need[i][j] {
                        flag = false;
                        break;
                    }
                }
                if flag {
                    no_new_finish = false;
                    finish_count += 1;
                    finish[i] = true;
                    for j in 0..resource_count{
                        work[j] += self.allocation[i][j];
                    }
                    break;
                }
            }
            if no_new_finish {
                break;
            }
        }
        if finish_count < thread_count {
            return false;
        }else{
            return true;
        }
    }


}