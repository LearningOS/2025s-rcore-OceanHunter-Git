# 总结实现的功能

## sys_spawn

首先fork父进程，生成一个新的进程，此时他与父进程基本相同，除了它的pid是新分配的，内核栈的地址也是新分配的。

然后是根据传入的文件名找到对应程序的elf，将子进程的一些数据修改为从elf中获取的数据，例如地址空间，上下文。

最后将子进程加入任务管理器。


## stride调度

原先的调度逻辑是FIFO，在一段时间内，每个任务运行的时间是平均的。

stride调度大致实现了在一段时间内，每个任务运行的时间与优先级成正比。

每个任务有stride和priority两个量，当一个任务运行时，它的stride会增加一个步长，步长的长度为BIG_STRIDE/priority(BIG_stride是一个较大的实数),在调度时总是运行stride最小的那个任务。


# 简答题

Q1:stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。实际情况是轮到 p1 执行吗？为什么？

A1:并不是，p2在执行一个时间片后，由于溢出导致p2的stride变为4小于了p1的stride，下一次仍然是p2运行。

Q2:我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明，在不考虑溢出的情况下, 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么STRIDE_MAX – STRIDE_MIN <= BigStride / 2。为什么？

A2:首先假设两个任务的stride都为0，任取一个任务先运行，由于优先级要求>=2,所以先运行的任务的的stride的范围为(1, BigStride/2]。此时的STRIDE_MAX - STRIDE_MIN <= BigStride / 2。接下来在另一个任务追平先运行的这个任务之前，先运行的任务的stride不会再增长，STRIDE_MAX - STRIDE_MIN 的值只会越来越小。追平时我们又可以看作两个任务的stride都为0。

Q3:已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

A3:
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 > other.0{
            if self.0 - other.0 > u64::MAX/2{
                return Some(Ordering::Less);
            }else{
                return Some(Ordering::Greater);
            }
        }else{
            if other.0 - self.0 > u64::MAX/2{
                return Some(Ordering::Greater);
            }else{
                return Some(Ordering::Less);
            }
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

 # 荣誉准则

 1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

 2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

 3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

 4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。