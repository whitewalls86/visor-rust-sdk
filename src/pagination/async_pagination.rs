use std::collections::VecDeque;

use futures::Stream;
use uuid::Uuid;

use crate::client::AsyncVisorClient;
use crate::error::VisorError;
use crate::models::dealers::{DealerFilter, DealerSummary};
use crate::models::listings::{ListingSummary, ListingsFilter};

struct ListingPageState<'a> {
    client: &'a AsyncVisorClient,
    filter: ListingsFilter,
    buffer: VecDeque<ListingSummary>,
    pages_fetched: usize,
    max_pages: Option<usize>,
    done: bool,
}

/// Returns a [`Stream`] of listing summaries, fetching pages until the API is
/// exhausted or `max_pages` pages have been fetched.
///
/// Items are yielded in API order. On an API or network error, the error is
/// yielded once and the stream terminates. `Some(0)` performs no request.
pub fn paginate_listings<'a>(
    client: &'a AsyncVisorClient,
    filter: ListingsFilter,
    max_pages: Option<usize>,
) -> impl Stream<Item = Result<ListingSummary, VisorError>> + 'a {
    futures::stream::unfold(
        ListingPageState {
            client,
            filter,
            buffer: VecDeque::new(),
            pages_fetched: 0,
            max_pages,
            done: false,
        },
        |mut s| async move {
            // Drain buffered items before checking done: the last page sets
            // done=true but all its items must still be yielded.
            if let Some(item) = s.buffer.pop_front() {
                return Some((Ok(item), s));
            }
            if s.done {
                return None;
            }
            if s.max_pages.is_some_and(|max| s.pages_fetched >= max) {
                return None;
            }
            match s.client.filter_listings(&s.filter).await {
                Err(e) => {
                    s.done = true;
                    Some((Err(e), s))
                }
                Ok(page) => {
                    s.pages_fetched += 1;
                    if page.data.is_empty() {
                        return None;
                    }
                    advance_offset(
                        &mut s.done,
                        &mut s.filter.offset,
                        page.pagination.next_offset,
                    );
                    s.buffer = page.data.into_iter().collect();
                    s.buffer.pop_front().map(|item| (Ok(item), s))
                }
            }
        },
    )
}

struct DealerPageState<'a> {
    client: &'a AsyncVisorClient,
    filter: DealerFilter,
    buffer: VecDeque<DealerSummary>,
    pages_fetched: usize,
    max_pages: Option<usize>,
    done: bool,
}

/// Returns a [`Stream`] of dealer summaries, fetching pages until the API is
/// exhausted or `max_pages` pages have been fetched.
///
/// Items are yielded in API order. On an API or network error, the error is
/// yielded once and the stream terminates. `Some(0)` performs no request.
pub fn paginate_dealers<'a>(
    client: &'a AsyncVisorClient,
    filter: DealerFilter,
    max_pages: Option<usize>,
) -> impl Stream<Item = Result<DealerSummary, VisorError>> + 'a {
    futures::stream::unfold(
        DealerPageState {
            client,
            filter,
            buffer: VecDeque::new(),
            pages_fetched: 0,
            max_pages,
            done: false,
        },
        |mut s| async move {
            if let Some(item) = s.buffer.pop_front() {
                return Some((Ok(item), s));
            }
            if s.done {
                return None;
            }
            if s.max_pages.is_some_and(|max| s.pages_fetched >= max) {
                return None;
            }
            match s.client.search_dealers(&s.filter).await {
                Err(e) => {
                    s.done = true;
                    Some((Err(e), s))
                }
                Ok(page) => {
                    s.pages_fetched += 1;
                    if page.data.is_empty() {
                        return None;
                    }
                    advance_offset(
                        &mut s.done,
                        &mut s.filter.offset,
                        page.pagination.next_offset,
                    );
                    s.buffer = page.data.into_iter().collect();
                    s.buffer.pop_front().map(|item| (Ok(item), s))
                }
            }
        },
    )
}

struct DealerInventoryPageState<'a> {
    client: &'a AsyncVisorClient,
    dealer_id: Uuid,
    filter: ListingsFilter,
    buffer: VecDeque<ListingSummary>,
    pages_fetched: usize,
    max_pages: Option<usize>,
    done: bool,
}

/// Returns a [`Stream`] of listing summaries for a dealer's inventory, fetching
/// pages until the API is exhausted or `max_pages` pages have been fetched.
///
/// Items are yielded in API order. On an API or network error, the error is
/// yielded once and the stream terminates. `Some(0)` performs no request.
pub fn paginate_dealer_inventory<'a>(
    client: &'a AsyncVisorClient,
    dealer_id: Uuid,
    filter: ListingsFilter,
    max_pages: Option<usize>,
) -> impl Stream<Item = Result<ListingSummary, VisorError>> + 'a {
    futures::stream::unfold(
        DealerInventoryPageState {
            client,
            dealer_id,
            filter,
            buffer: VecDeque::new(),
            pages_fetched: 0,
            max_pages,
            done: false,
        },
        |mut s| async move {
            if let Some(item) = s.buffer.pop_front() {
                return Some((Ok(item), s));
            }
            if s.done {
                return None;
            }
            if s.max_pages.is_some_and(|max| s.pages_fetched >= max) {
                return None;
            }
            match s.client.dealer_inventory(s.dealer_id, &s.filter).await {
                Err(e) => {
                    s.done = true;
                    Some((Err(e), s))
                }
                Ok(page) => {
                    s.pages_fetched += 1;
                    if page.data.is_empty() {
                        return None;
                    }
                    advance_offset(
                        &mut s.done,
                        &mut s.filter.offset,
                        page.pagination.next_offset,
                    );
                    s.buffer = page.data.into_iter().collect();
                    s.buffer.pop_front().map(|item| (Ok(item), s))
                }
            }
        },
    )
}

/// Updates `current_offset` to `next_offset` when it advances, or sets `done`
/// if `next_offset` is absent or non-advancing (guards against infinite loops).
fn advance_offset(done: &mut bool, current_offset: &mut u32, next_offset: Option<i32>) {
    match next_offset {
        Some(next) if next >= 0 && (next as u32) > *current_offset => {
            *current_offset = next as u32;
        }
        _ => *done = true,
    }
}
