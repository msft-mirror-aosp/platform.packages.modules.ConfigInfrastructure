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

import static com.android.server.deviceconfig.ServiceResourcesHelper.RESOURCES_PACKAGE_SUFFIX;
import static com.google.common.truth.Truth.assertThat;

import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import android.content.Context;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;

import androidx.test.filters.SmallTest;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@SmallTest
public class ServiceResourcesHelperTest {
    @Rule public MockitoRule mockitoRule = MockitoJUnit.rule();

    @Mock private Context mContext;
    @Mock private PackageManager mPackageManager;

    @Before
    public void setUp() throws Exception {
        ServiceResourcesHelper.setInstanceForTest(null);
        when(mContext.getPackageManager()).thenReturn(mPackageManager);
    }

    @Test
    public void getResourcesPackageName_singleValidPackage_returnsThePackage() {
        mockInstalledPackages(
                PackageManager.MATCH_SYSTEM_ONLY,
                "random.package.name", "abc" + RESOURCES_PACKAGE_SUFFIX, "a.b.c");

        assertThat(ServiceResourcesHelper.get(mContext).getResourcesPackageName())
                .isEqualTo(Optional.of("abc" + RESOURCES_PACKAGE_SUFFIX));
    }

    @Test
    public void getResourcesPackageName_multipleValidPackages_returnsFirstOne() {
        mockInstalledPackages(
                PackageManager.MATCH_SYSTEM_ONLY,
                "abc" + RESOURCES_PACKAGE_SUFFIX, "a" + RESOURCES_PACKAGE_SUFFIX);

        assertThat(ServiceResourcesHelper.get(mContext).getResourcesPackageName())
                .isEqualTo(Optional.of("abc" + RESOURCES_PACKAGE_SUFFIX));
    }

    @Test
    public void getResourcesPackageName_usesOnlySystemPackages() {
        mockInstalledPackages(
                PackageManager.MATCH_SYSTEM_ONLY,
                "random.package.name", "a.b.c");
        // Although one of these packages has a valid resource package name format, it's not a
        // system package, so it should not be returned from `getResourcesPackageName`.
        mockInstalledPackages(
                PackageManager.MATCH_ALL,
                "random.package.name", "a.b.c", "abc" + RESOURCES_PACKAGE_SUFFIX);

        assertThat(ServiceResourcesHelper.get(mContext).getResourcesPackageName())
                .isEqualTo(Optional.empty());
    }

    @Test
    public void resourcePackageFetchedOnlyOnce() {
        ServiceResourcesHelper.get(mContext).getResourcesPackageName();
        ServiceResourcesHelper.get(mContext).getResourcesPackageName();

        verify(mPackageManager, times(1)).getInstalledPackages(anyInt());
    }

    private PackageInfo createPackageInfo(String packageName) {
        PackageInfo info = new PackageInfo();
        info.packageName = packageName;
        return info;
    }

    private void mockInstalledPackages(int packageManagerFlag, String... packageNames) {
        List<PackageInfo> info = new ArrayList<>();
        for (String packageName : packageNames) {
            info.add(createPackageInfo(packageName));
        }
        when(mPackageManager.getInstalledPackages(packageManagerFlag)).thenReturn(info);
    }
}
