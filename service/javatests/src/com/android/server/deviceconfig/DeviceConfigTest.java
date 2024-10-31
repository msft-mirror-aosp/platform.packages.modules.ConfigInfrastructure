/*
 * Copyright (C) 2024 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.android.server.deviceconfig;

import android.platform.test.annotations.RequiresFlagsEnabled;
import android.platform.test.flag.junit.CheckFlagsRule;
import android.platform.test.flag.junit.DeviceFlagsValueProvider;
import android.provider.flags.Flags;
import android.provider.DeviceConfig;
import android.provider.DeviceConfig.OnPropertiesChangedListener;
import android.provider.DeviceConfig.Properties;
import android.util.Log;

import com.google.common.truth.Expect;

import org.junit.Rule;
import org.junit.Test;

import java.io.IOException;
import java.io.PrintWriter;
import java.io.StringWriter;

public final class DeviceConfigTest {

    private static final String TAG = DeviceConfigTest.class.getSimpleName();

    private static final String NAMESPACE_A = "A Space has no name";
    private static final String NAMESPACE_B = "B Space has no name";

    private static final String DUMP_PREFIX = "..";

    @Rule public final Expect expect = Expect.create();
    @Rule public final CheckFlagsRule checkFlagsRule =
            DeviceFlagsValueProvider.createCheckFlagsRule();

    @Test
    @RequiresFlagsEnabled(Flags.FLAG_DUMP_IMPROVEMENTS)
    public void testDump_empty() throws Exception {
        String dump = dump();

        expect.withMessage("dump()").that(dump).isEqualTo(DUMP_PREFIX
                + "0 listeners for 0 namespaces:\n");
    }

    @Test
    @RequiresFlagsEnabled(Flags.FLAG_DUMP_IMPROVEMENTS)
    public void testDump_withListeners() throws Exception {
        var listener1 = new TestOnPropertiesChangedListener();
        var listener2 = new TestOnPropertiesChangedListener();
        var listener3 = new TestOnPropertiesChangedListener();

        DeviceConfig.addOnPropertiesChangedListener(NAMESPACE_A, Runnable::run, listener1);
        DeviceConfig.addOnPropertiesChangedListener(NAMESPACE_A, Runnable::run, listener2);
        DeviceConfig.addOnPropertiesChangedListener(NAMESPACE_A, Runnable::run, listener3);
        // Next call will remove listener1 from NAMESPACE_A
        DeviceConfig.addOnPropertiesChangedListener(NAMESPACE_B, Runnable::run, listener1);

        try {
            String dump = dump();

            expect.withMessage("dump()").that(dump).isEqualTo(DUMP_PREFIX
                    + "3 listeners for 2 namespaces:\n"
                    + DUMP_PREFIX + NAMESPACE_A + ": 2 listeners\n"
                    + DUMP_PREFIX + DUMP_PREFIX + listener2 + "\n"
                    + DUMP_PREFIX + DUMP_PREFIX + listener3 + "\n"
                    + DUMP_PREFIX + NAMESPACE_B + ": 1 listeners\n"
                    + DUMP_PREFIX + DUMP_PREFIX + listener1 + "\n"
                    );
        } finally {
            DeviceConfig.removeOnPropertiesChangedListener(listener1);
            DeviceConfig.removeOnPropertiesChangedListener(listener2);
            DeviceConfig.removeOnPropertiesChangedListener(listener3);
        }
    }

    private String dump(String...args) throws IOException {
        try (StringWriter sw = new StringWriter()) {
            PrintWriter pw = new PrintWriter(sw);

            DeviceConfig.dump(/* fd= */ null, pw, DUMP_PREFIX, args);

            pw.flush();
            String dump = sw.toString();

            Log.v(TAG, "dump() output\n" + dump);

            return dump;
        }
    }

    private static final class TestOnPropertiesChangedListener
            implements OnPropertiesChangedListener {

        private static int sNextId;

        private final int mId = ++sNextId;

        @Override
        public void onPropertiesChanged(Properties properties) {
            throw new UnsupportedOperationException("Not used in any test (yet?)");
        }

        @Override
        public String toString() {
            return "TestOnPropertiesChangedListener#" + mId;
        }
    }
}
