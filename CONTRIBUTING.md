# Contributing to Rair Core


Thank you for your interest in making Rair Core a better library! We are glad that someone somewhere
shares our enthusiasm and passion for creating state of art reverse engineering platform.


First: if you're unsure or *afraid* of anything, just ask or submit the issue or pull request anyway.
You won't be yelled at for giving it your best effort. The worst that can happen is that you'll be
politely asked to change something. We appreciate any sort of contributions and don't want a wall
of rules to get in the way of that.

However, by trying to follow the guidelines in this file, you would make it easier for us to review
your pull request and merge it quickly with little to no extra effort on your end.

## But why too many guidelines, I just made a POC and want it merged

In our experience, we believe that people are ultimately more excited towards adding new feature rather
than fixing a bug in an already existing feature. Maintenance is boring business, so we try to make sure
that every line of code added to rair-core repository would require very little maintenance and is more
likely to be bugs free.


Before submitting a pull request please make sure to:

a) Run `cargo fmt` on all your code

b) Run `cargo clippy` and verify that no warnings are reported

c) Any user-facing API should have at least documentation. It would be better if doctests are provided as well

d) Please split your code into self-contained commits. Ideally, each commit should do one and only one thing.

e) Commits titles should descriptive and in a commanding tone. It should feel like you are giving the commit
an order to do something like `Convert rair-core into lock-free, mutex free, concurrent data structure`.

f) All newly added code must be well tested. We use code coverage as a metric. A general rule of thumb is that coverage
should never go down, unless there is a good reason (bug in the coverage tool is the only good reason we ever
encountered).

