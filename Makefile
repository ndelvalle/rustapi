test:
	# Every test runs against its own temporary database, created on the server
	# that DATABASE_URL points to.

	# By default the Rust test harness hides output from test execution to keep
	# results readable. The nocapture flag disables that behavior.

	cargo test -- \
  		--nocapture \
  		--color=always
