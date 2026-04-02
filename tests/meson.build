# Copyright © 2026 Valve Corporation
#
# SPDX-License-Identifier: BSD-3-Clause

# Start and use a separate message bus for the tests,
# to not interfere with the current's session message bus.
dbus_run_session = find_program('dbus-run-session', required: true)

test(
    'integration tests',
    find_program('test-fixture.sh'),
    args: [dbus_run_session.full_path(), cargo.full_path(), 'test'],
    depends: [cargo_build],
    env: cargo_env + ['XDG_CURRENT_DESKTOP=gamescope'],
)
