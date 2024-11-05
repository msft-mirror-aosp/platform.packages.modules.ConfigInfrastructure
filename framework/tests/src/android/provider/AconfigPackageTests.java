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

package android.configinfrastructure.aconfig.test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

import android.aconfig.DeviceProtos;
import android.aconfig.nano.Aconfig.parsed_flag;
import android.aconfig.storage.AconfigStorageReadAPI;
import android.aconfig.storage.FlagReadContext;
import android.aconfig.storage.PackageReadContext;
import android.configinfrastructure.aconfig.AconfigPackage;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

import java.io.IOException;
import java.nio.MappedByteBuffer;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@RunWith(JUnit4.class)
public class AconfigPackageTests {
    @Test
    public void testExternalAconfigPackageInstance() throws IOException {
        List<parsed_flag> flags = DeviceProtos.loadAndParseFlagProtos();
        Map<String, AconfigPackage> readerMap = new HashMap<>();
        String mapPath = "/metadata/aconfig/maps/";
        String flagsPath = "/metadata/aconfig/boot/";

        for (parsed_flag flag : flags) {

            String container = flag.container;
            String packageName = flag.package_;
            String flagName = flag.name;

            MappedByteBuffer packageMap =
                    AconfigStorageReadAPI.mapStorageFile(mapPath + container + ".package.map");
            MappedByteBuffer flagMap =
                    AconfigStorageReadAPI.mapStorageFile(mapPath + container + ".flag.map");
            MappedByteBuffer flagValList =
                    AconfigStorageReadAPI.mapStorageFile(flagsPath + container + ".val");

            PackageReadContext packageContext =
                    AconfigStorageReadAPI.getPackageReadContext(packageMap, packageName);

            FlagReadContext flagContext =
                    AconfigStorageReadAPI.getFlagReadContext(
                            flagMap, packageContext.mPackageId, flagName);

            boolean rVal =
                    AconfigStorageReadAPI.getBooleanFlagValue(
                            flagValList,
                            packageContext.mBooleanStartIndex + flagContext.mFlagIndex);

            AconfigPackage reader = readerMap.get(packageName);
            if (reader == null) {
                reader = AconfigPackage.load(packageName);
                readerMap.put(packageName, reader);
            }
            boolean jVal = reader.getBooleanFlagValue(flagName, false);

            assertEquals(rVal, jVal);
        }
    }

    @Test
    public void testAconfigPackage_load_withError() {
        // load fake package
        AconfigPackage p = AconfigPackage.load("fake_package");
        assertNotNull(p);
        assertFalse(p.getBooleanFlagValue("fake_flag", false));
        assertTrue(p.getBooleanFlagValue("fake_flag", true));
    }
}
