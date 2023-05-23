
test:	
	# Tests depend on the database state. The database is cleanup before each
	# test, this is why tests needs to be run sequentially.

	# By default the Rust test harness hides output from test execution to keep
	# results readable. The nocapture flag disables that behavior.

	cargo test -- \
  		--test-threads=1 \
  		--nocapture \
  		--color=always
