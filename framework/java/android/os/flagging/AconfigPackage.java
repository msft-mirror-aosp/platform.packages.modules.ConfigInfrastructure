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

package android.os.flagging;

import static android.provider.flags.Flags.FLAG_NEW_STORAGE_PUBLIC_API;

import android.aconfig.storage.StorageFileProvider;
import android.annotation.FlaggedApi;
import android.annotation.NonNull;

/**
 * An {@code aconfig} package containing the enabled state of its flags.
 *
 * <p><strong>Note: this is intended only to be used by generated code. To determine if a given flag
 * is enabled in app code, the generated android flags should be used.</strong>
 *
 * <p>This class is used to read the flag from Aconfig Package.Each instance of this class will
 * cache information related to one package. To read flags from a different package, a new instance
 * of this class should be {@link #load loaded}.
 */
@FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
public class AconfigPackage {

    private static final String[] PLATFORM_CONTAINERS = {"system", "product", "vendor"};

    private final PlatformAconfigPackageInternal mPlatformPackage;
    private final AconfigPackageInternal mAconfigPackage;

    private AconfigPackage(
            PlatformAconfigPackageInternal pPackage, AconfigPackageInternal aPackage) {
        mPlatformPackage = pPackage;
        mAconfigPackage = aPackage;
    }

    /**
     * Loads an Aconfig Package from Aconfig Storage.
     *
     * <p>This method attempts to load the specified Aconfig package.
     *
     * @param packageName The name of the Aconfig package to load.
     * @return An instance of {@link AconfigPackage}, which may be empty if the package is not found
     *     in the container.
     * @throws AconfigStorageReadException if there is an error reading from Aconfig Storage, such
     *     as if the storage system is not found, the package is not found, or there is an error
     *     reading the storage file. The specific error code can be obtained using {@link
     *     AconfigStorageReadException#getErrorCode()}.
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public static @NonNull AconfigPackage load(@NonNull String packageName) {
        StorageFileProvider fileProvider = StorageFileProvider.getDefaultProvider();
        // First try to load from platform containers.
        for (String container : PLATFORM_CONTAINERS) {
            PlatformAconfigPackageInternal pPackage =
                    PlatformAconfigPackageInternal.load(container, packageName);
            if (pPackage.getException() == null) {
                return new AconfigPackage(pPackage, null);
            }
        }

        // If not found in platform containers, search all package map files.
        for (String container : fileProvider.listContainers(PLATFORM_CONTAINERS)) {
            AconfigPackageInternal aPackage = AconfigPackageInternal.load(container, packageName);
            if (aPackage.getException() == null) {
                return new AconfigPackage(null, aPackage);
            }
        }

        // Package not found.
        throw new AconfigStorageReadException(
                AconfigStorageReadException.ERROR_PACKAGE_NOT_FOUND,
                "package " + packageName + " cannot be found on the device");
    }

    /**
     * Retrieves the value of a boolean flag.
     *
     * <p>This method retrieves the value of the specified flag. If the flag exists within the
     * loaded Aconfig Package, its value is returned. Otherwise, the provided `defaultValue` is
     * returned.
     *
     * @param flagName The name of the flag (excluding any package name prefix).
     * @param defaultValue The value to return if the flag is not found.
     * @return The boolean value of the flag, or `defaultValue` if the flag is not found.
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public boolean getBooleanFlagValue(@NonNull String flagName, boolean defaultValue) {
        if (mPlatformPackage != null) {
            return mPlatformPackage.getBooleanFlagValue(flagName, defaultValue);
        }
        return mAconfigPackage.getBooleanFlagValue(flagName, defaultValue);
    }
}
