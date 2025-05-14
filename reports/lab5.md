# 总结实现的功能

## 银行家算法检测是否会发生死锁

Banker结构体包含三个向量：

- `available[j]` 用于存储当前进程第j个资源的剩余数量

- `allocation[i][j]` 用于保存已经分配给第i个线程第j个资源的数量

- `need[i][j]` 用于保存第i个线程发起对第j个资源请求的数量

当线程请求一个锁的时候，会发送一个请求，然后银行家算法会判断这个请求是否可以。

在判断算法中有两个向量`work`和`finish`，`work[j]` 用于表示进程第j个资源的剩余数量，在初始的时候`work` = `available`，`finish[i]` 表示第i个线程是否完成，初始全为false。

遍历所有还没有完成的线程，如果该线程`i`的对于所有资源的请求小于当前`work`向量中剩余的资源，则认为该线程能完成，则将该线程的`finish[i]`设置为true，并返还已经分配的资源（`work += allocation[i]`）, 直到所有线程都完成或者该轮循环没有新完成的线程，则退出循环。如果完成的线程数小于总线程数则认为这个请求是不安全的，驳回这个请求。如果所有线程完成则继续进行，获取锁。

在实际获得锁的时候会将`need`的值加入到`allocation`, 并减少`available`中的值。

在实际释放锁的时候会减少`allocation`中对应的值，并增加`available`中对应的值。


# 简答题

Q1:在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

A1:
需要回收所有线程的`TaskUserRes`和进程的地址空间。子进程不用回收但是需要全部挂到`INITPROC`进程下。

其他线程的 TaskControlBlock 在进程的tasks中被引用，在TaskManager中的ready_queue和stop_task中被引用，还有在条件变量和信号量的阻塞队列中被引用，processor中的current。这些引用在主线程被回收的时候，这些引用都会取消引用，TaskControlBlock因为引用计数变为0会自动释放。

```rust
impl Mutex for Mutex1 {
    fn lock(&self) {
        loop {
            let mut mutex_inner = self.inner.exclusive_access();
            if mutex_inner.locked {
                mutex_inner.wait_queue.push_back(current_task().unwrap());
                drop(mutex_inner);
                block_current_and_run_next();
            } else {
                mutex_inner.locked = true;
                break;
            }
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn lock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```
Q2:对比以下两种 Mutex 中的实现，二者有什么区别？这些区别可能会导致什么问题？

A2:
第一种在上一个线程释放锁的时候是将锁的状态直接修改为没有被锁，然后唤醒一个线程，这个被唤醒的线程需要自己去竞争这个锁，如果没有竞争到还是会被加入到阻塞队列。
第二种在释放锁的时候就没有修改锁的状态，让被唤醒的线程不需要竞争就直接获得了锁，如果没有被阻塞的线程才将锁的状态改为没有被锁。

但是实际上并不能直接运行唤醒的线程，因为他只是被加入tasks并不是直接运行这个被唤醒的线程，所以在此期间由于锁的状态一直都是被锁了，直到新唤醒的这个线程被调度到了完成后才能释放锁，可能在运行的效率上不如第一种实现

 # 荣誉准则

 1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

 2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

 3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

 4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。