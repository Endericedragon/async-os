use ak_futures_bounded::{FuturesMap, PushError};
use ak_futures_timer::Delay;
use core::future::{pending, poll_fn, ready};
use core::task::Context;
use core::time::Duration;
use futures_util::task::noop_waker_ref;

pub fn cannot_push_more_than_capacity_tasks() {
    let mut futures = FuturesMap::new(Duration::from_secs(10), 1);

    assert!(futures.try_push("ID_1", ready(())).is_ok());
    matches!(
        futures.try_push("ID_2", ready(())),
        Err(PushError::BeyondCapacity(_))
    );
}

pub fn cannot_push_the_same_id_few_times() {
    let mut futures = FuturesMap::new(Duration::from_secs(10), 5);

    assert!(futures.try_push("ID", ready(())).is_ok());
    matches!(
        futures.try_push("ID", ready(())),
        Err(PushError::Replaced(_))
    );
}

pub async fn futures_timeout() {
    let mut futures = FuturesMap::new(Duration::from_millis(100), 1);

    let _ = futures.try_push("ID", pending::<()>());
    Delay::new(Duration::from_millis(150)).await;
    let (_, result) = poll_fn(|cx| futures.poll_unpin(cx)).await;

    assert!(result.is_err())
}

pub fn resources_of_removed_future_are_cleaned_up() {
    let mut futures = FuturesMap::new(Duration::from_millis(100), 1);

    let _ = futures.try_push("ID", pending::<()>());
    futures.remove("ID");

    let poll = futures.poll_unpin(&mut Context::from_waker(noop_waker_ref()));
    assert!(poll.is_pending());

    assert_eq!(futures.len(), 0);
}

pub async fn replaced_pending_future_is_polled() {
    let mut streams = FuturesMap::new(Duration::from_millis(100), 3);

    // async_std::sync::

    let (_tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    let _ = streams.try_push("ID1", rx1);
    let _ = streams.try_push("ID2", rx2);

    let _ = tx2.send(2);
    let (id, res) = poll_fn(|cx| streams.poll_unpin(cx)).await;
    assert_eq!(id, "ID2");
    assert_eq!(res.unwrap().unwrap(), 2);

    let (new_tx1, new_rx1) = oneshot::channel();
    let replaced = streams.try_push("ID1", new_rx1);
    assert!(matches!(replaced.unwrap_err(), PushError::Replaced(_)));

    let _ = new_tx1.send(4);
    let (id, res) = poll_fn(|cx| streams.poll_unpin(cx)).await;

    assert_eq!(id, "ID1");
    assert_eq!(res.unwrap().unwrap(), 4);
}
