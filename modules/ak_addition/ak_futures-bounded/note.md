# Futures-bounded 改造计划

该库有两个依赖：`futures-util` 和 `futures-timer` 。其中，前者本身就支持 `no_std` ，而后者也已经完成改造。此时，需要修改 `Cargo.toml` 文件，将上述依赖分别改写为：

```toml
# 开启futures-util的no_std支持，并启用必要的feature
futures-util = { version = "0.3.30", default-features = false, features = [
    "alloc"
] }
# 直接使用我们改好的futures-timer库
futures-timer = { path = "../async_futures-timer", package = "async_futures-timer" }
```

随后的修改则极为简单，以 `src/futures_map.rs` 为例：

```rust
use std::future::Future;
// 更改为如下内容
use core::future::Future;
```

即：用core和std中已经存在的trait去替换std中的trait。在改写过程中唯一一个无法这样做的是 `std::collections::HashMap` ，此时我们可以请出 `my_std::collections::HashMap` 加以替代。