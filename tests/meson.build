# Copyright © 2026 Valve Corporation
#
# SPDX-License-Identifier: BSD-3-Clause

# Start and use a separate message bus for the tests,
# to not interfere with the current's session message bus.
dbus_run_session = find_program('dbus-run-session', required: true)

test(
    'integration tests',
    find_program('test-fixture.sh'),
    args: [
        dbus_run_session.full_path(),
        cargo.full_path(),
        'test',
        '--test', 'integration',
    ],
    env: cargo_env + ['XDG_CURRENT_DESKTOP=gamescope'],
    # Integration tests are compiled only when `cargo test` is run, which has
    # the unfortunate side effect that the build time is counted as part of the
    # run time for the tests. Meson's default (30 seconds) can be too short to
    # build AND run the tests.
    timeout: 120,
)
