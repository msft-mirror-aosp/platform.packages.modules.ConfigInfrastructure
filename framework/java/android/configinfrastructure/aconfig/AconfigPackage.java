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

package android.configinfrastructure.aconfig;

import static android.provider.flags.Flags.FLAG_NEW_STORAGE_PUBLIC_API;

import android.aconfig.storage.AconfigPackageImpl;
import android.aconfig.storage.AconfigStorageException;
import android.aconfig.storage.StorageFileProvider;
import android.annotation.FlaggedApi;
import android.annotation.NonNull;

import java.nio.file.FileSystemNotFoundException;

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

    private AconfigPackageImpl impl;

    private AconfigPackage() {}

    private static AconfigPackage EMPTY = new AconfigPackage();

    /**
     * Loads an Aconfig Package from Aconfig Storage.
     *
     * <p>This method attempts to load the specified Aconfig package. If the package is not found in
     * Aconfig Storage, an empty instance of {@link AconfigPackage} is returned. This empty instance
     * is not backed by a real Aconfig package in storage, meaning it will not contain any
     * configuration data.
     *
     * @throws AconfigStorageReadException If the package is not found in the container.
     * @throws FileSystemNotFoundException If Aconfig Storage is not available on the device.
     * @param packageName The name of the Aconfig package to load.
     * @return An instance of {@link AconfigPackage}, which may be empty if the package is not
     *     found in the container.
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public static @NonNull AconfigPackage load(@NonNull String packageName) {
        AconfigPackage aPackage = new AconfigPackage();
        aPackage.impl = new AconfigPackageImpl();
        int code = 0;
        try {
            code = aPackage.impl.load(packageName, StorageFileProvider.getDefaultProvider());
        } catch (AconfigStorageException e) {
            throw new AconfigStorageReadException(e);
        }
        switch (code) {
            case AconfigPackageImpl.ERROR_NEW_STORAGE_SYSTEM_NOT_FOUND:
                throw new FileSystemNotFoundException(
                        "Aconfig new storage is not found on this device");
            case AconfigPackageImpl.ERROR_PACKAGE_NOT_FOUND:
                return EMPTY;
            default:
                // it won't have container not found error
        }
        return aPackage;
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
        if (this.equals(EMPTY)) {
            return defaultValue;
        }
        return impl.getBooleanFlagValue(flagName, defaultValue);
    }
}
