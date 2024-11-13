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

import android.aconfig.DeviceProtos;
import android.aconfig.nano.Aconfig.parsed_flag;
import android.aconfig.storage.FlagTable;
import android.aconfig.storage.FlagValueList;
import android.aconfig.storage.PackageTable;
import android.aconfig.storage.StorageFileProvider;
import android.os.flagging.AconfigPackageLocal;
import android.os.flagging.AconfigStorageReadException;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@RunWith(JUnit4.class)
public class AconfigPackageLocalTests {
    @Test
    public void testExternalAconfigPackageLocalInstance() throws IOException {
        List<parsed_flag> flags = DeviceProtos.loadAndParseFlagProtos();
        Map<String, AconfigPackageLocal> readerMap = new HashMap<>();
        StorageFileProvider fp = StorageFileProvider.getDefaultProvider();

        for (parsed_flag flag : flags) {

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

            AconfigPackageLocal reader = readerMap.get(packageName);
            if (reader == null) {
                reader = AconfigPackageLocal.load(container, packageName, fingerprint);
                readerMap.put(packageName, reader);
            }
            boolean jVal = reader.getBooleanFlagValue(fNode.getFlagIndex());

            assertEquals(rVal, jVal);
        }
    }

    @Test
    public void testAconfigPackage_load_withError() throws IOException {
        AconfigStorageReadException e;
        // container not found fake_container
        e =
                assertThrows(
                        AconfigStorageReadException.class,
                        () -> AconfigPackageLocal.load("fake_container", "fake_package", 0));
        assertEquals(AconfigStorageReadException.ERROR_CANNOT_READ_STORAGE_FILE, e.getErrorCode());

        // package not found
        e =
                assertThrows(
                        AconfigStorageReadException.class,
                        () -> AconfigPackageLocal.load("system", "fake_container", 0));
        assertEquals(AconfigStorageReadException.ERROR_PACKAGE_NOT_FOUND, e.getErrorCode());

        // fingerprint doesn't match

        List<parsed_flag> flags = DeviceProtos.loadAndParseFlagProtos();
        StorageFileProvider fp = StorageFileProvider.getDefaultProvider();

        parsed_flag flag = flags.get(0);

        String container = flag.container;
        String packageName = flag.package_;

        PackageTable pTable = fp.getPackageTable(container);
        PackageTable.Node pNode = pTable.get(packageName);
        long fingerprint = pNode.getPackageFingerprint();
        if (pNode.hasPackageFingerprint()) {
            e =
                    assertThrows(
                            AconfigStorageReadException.class,
                            () ->
                                    AconfigPackageLocal.load(
                                            container, packageName, fingerprint + 1));
            assertEquals(
                    AconfigStorageReadException.ERROR_FILE_FINGERPRINT_MISMATCH, e.getErrorCode());
        }
    }
}
