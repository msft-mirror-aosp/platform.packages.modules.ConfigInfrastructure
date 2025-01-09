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

package android.os.flagging.test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

import android.aconfig.DeviceProtosTestUtil;
import android.aconfig.nano.Aconfig;
import android.aconfig.nano.Aconfig.parsed_flag;
import android.aconfig.storage.AconfigStorageException;
import android.aconfig.storage.FlagTable;
import android.aconfig.storage.FlagValueList;
import android.aconfig.storage.PackageTable;
import android.aconfig.storage.StorageFileProvider;
import android.os.flagging.AconfigPackageInternal;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@RunWith(JUnit4.class)
public class AconfigPackageInternalTests {
    @Test
    public void testAconfigPackageInternal_load() throws IOException {
        List<parsed_flag> flags = DeviceProtosTestUtil.loadAndParseFlagProtos();
        Map<String, AconfigPackageInternal> readerMap = new HashMap<>();
        StorageFileProvider fp = StorageFileProvider.getDefaultProvider();

        for (parsed_flag flag : flags) {
            if (flag.permission == Aconfig.READ_ONLY && flag.state == Aconfig.DISABLED) {
                continue;
            }
            String container = flag.container;
            String packageName = flag.package_;
            String flagName = flag.name;

            PackageTable pTable = fp.getPackageTable(container);
            PackageTable.Node pNode = pTable.get(packageName);
            FlagTable fTable = fp.getFlagTable(container);
            FlagTable.Node fNode = fTable.get(pNode.getPackageId(), flagName);
            FlagValueList fList = fp.getFlagValueList(container);

            int index = pNode.getBooleanStartIndex() + fNode.getFlagIndex();
            boolean rVal = fList.getBoolean(index);

            long fingerprint = pNode.getPackageFingerprint();

            AconfigPackageInternal reader = readerMap.get(packageName);
            if (reader == null) {
                reader = AconfigPackageInternal.load(packageName, fingerprint);
                readerMap.put(packageName, reader);
            }
            boolean jVal = reader.getBooleanFlagValue(fNode.getFlagIndex());

            assertEquals(rVal, jVal);
        }
    }

    @Test
    public void testAconfigPackageInternal_load_withError() throws IOException {
        // package not found
        AconfigStorageException e =
                assertThrows(
                        AconfigStorageException.class,
                        () -> AconfigPackageInternal.load("fake_package", 0));
        assertEquals(AconfigStorageException.ERROR_PACKAGE_NOT_FOUND, e.getErrorCode());

        // fingerprint doesn't match
        List<parsed_flag> flags = DeviceProtosTestUtil.loadAndParseFlagProtos();
        StorageFileProvider fp = StorageFileProvider.getDefaultProvider();

        parsed_flag flag = flags.get(0);

        String container = flag.container;
        String packageName = flag.package_;

        PackageTable pTable = fp.getPackageTable(container);
        PackageTable.Node pNode = pTable.get(packageName);
        if (pNode.hasPackageFingerprint()) {
            long fingerprint = pNode.getPackageFingerprint();
            e =
                    assertThrows(
                            AconfigStorageException.class,
                            () -> AconfigPackageInternal.load(packageName, fingerprint + 1));
            assertEquals(AconfigStorageException.ERROR_FILE_FINGERPRINT_MISMATCH, e.getErrorCode());
        }
    }
}
