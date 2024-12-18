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

import android.annotation.Nullable;
import android.content.Context;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.util.Log;

import com.android.internal.annotations.VisibleForTesting;

import java.util.List;
import java.util.Optional;

/**
 * Helper class for the service resources package.
 *
 * @hide
 */
public class ServiceResourcesHelper {
    private static final String TAG = "ServiceResourcesHelper";

    @VisibleForTesting
    static final String RESOURCES_PACKAGE_SUFFIX =
            ".android.server.deviceconfig.resources";

    @Nullable private static ServiceResourcesHelper sHelper; // singleton.

    /** Provides an instance of the helper. */
    public static ServiceResourcesHelper get(Context context) {
        if (sHelper == null) {
            sHelper = new ServiceResourcesHelper(context);
        }
        return sHelper;
    }

    /** Sets the helper instance for testing purposes. */
    @VisibleForTesting
    public static void setInstanceForTest(ServiceResourcesHelper helper) {
        sHelper = helper;
    }

    private final Optional<String> mServiceResourcesPackageName;

    @VisibleForTesting
    public ServiceResourcesHelper(Context context) {
        mServiceResourcesPackageName =
                getSystemPackageNameBySuffix(context, RESOURCES_PACKAGE_SUFFIX);
    }

    /**
     * Returns an {@link Optional} containing the service resources package name, or an empty
     * {@link Optional} if the package cannot be found.
     */
    public Optional<String> getResourcesPackageName() {
        return mServiceResourcesPackageName;
    }

    private Optional<String> getSystemPackageNameBySuffix(Context context, String suffix) {
        PackageManager packageManager = context.getPackageManager();
        List<PackageInfo> installedSystemPackages =
                packageManager.getInstalledPackages(PackageManager.MATCH_SYSTEM_ONLY);

        String serviceResourcesPackageName = null;
        for (PackageInfo packageInfo : installedSystemPackages) {
            final String packageName = packageInfo.packageName;
            if (packageInfo.packageName.endsWith(RESOURCES_PACKAGE_SUFFIX)) {
                Log.i(TAG, "Found service resource package: " + packageName);
                if (serviceResourcesPackageName != null) {
                    Log.w(TAG, "Multiple service resource packages found");
                } else {
                    serviceResourcesPackageName = packageName;
                }
            }
        }

        return Optional.ofNullable(serviceResourcesPackageName);
    }
}