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

import android.aconfig.storage.AconfigStorageException;
import android.aconfig.storage.FlagValueList;
import android.aconfig.storage.PackageTable;
import android.aconfig.storage.StorageFileProvider;
import android.annotation.NonNull;
import android.compat.annotation.UnsupportedAppUsage;
import android.os.StrictMode;

/**
 * An {@code aconfig} package containing the enabled state of its flags.
 *
 * <p><strong>Note: this is intended only to be used by generated code. To determine if a given flag
 * is enabled in app code, the generated android flags should be used.</strong>
 *
 * <p>This class is not part of the public API and should be used by Acnofig Flag internally </b> It
 * is intended for internal use only and will be changed or removed without notice.
 *
 * <p>This class is used to read the flag from Aconfig Package.Each instance of this class will
 * cache information related to one package. To read flags from a different package, a new instance
 * of this class should be {@link #load loaded}.
 *
 * @hide
 */
public class AconfigPackageLocal {

    private final FlagValueList mFlagValueList;
    private final int mPackageBooleanStartOffset;

    private AconfigPackageLocal(
            @NonNull FlagValueList flagValueList, int packageBooleanStartOffset) {
        this.mFlagValueList = flagValueList;
        this.mPackageBooleanStartOffset = packageBooleanStartOffset;
    }

    /**
     * Loads an Aconfig package from the specified container and verifies its fingerprint.
     *
     * <p>This method is intended for internal use only and may be changed or removed without
     * notice.
     *
     * <p>The fingerprint, generated at build time, ensures that the loaded package matches the
     * expected version. Providing a matching fingerprint optimizes flag retrieval by allowing the
     * use of direct indexing.
     *
     * @param container The name of the container.
     * @param packageName The name of the Aconfig package.
     * @param packageFingerprint The expected fingerprint of the package.
     * @return An instance of {@link AconfigPackageLocal} representing the loaded package.
     * @throws AconfigStorageReadException if there is an error reading from Aconfig Storage, if the
     *     fingerprint does not match, or if the specified container is a platform container. The
     *     specific error code can be got using {@link AconfigStorageReadException#getErrorCode()}.
     * @hide
     */
    @UnsupportedAppUsage
    public static @NonNull AconfigPackageLocal load(
            @NonNull String container, @NonNull String packageName, long packageFingerprint) {
        StrictMode.ThreadPolicy oldPolicy = StrictMode.allowThreadDiskReads();
        try {
            StorageFileProvider fileProvider = StorageFileProvider.getDefaultProvider();
            PackageTable.Node pNode = fileProvider.getPackageTable(container).get(packageName);

            if (pNode == null) {
                throw new AconfigStorageReadException(
                        AconfigStorageReadException.ERROR_PACKAGE_NOT_FOUND,
                        "package "
                                + packageName
                                + " in container "
                                + container
                                + " cannot be found on the device");
            }

            if (pNode.hasPackageFingerprint()
                    && packageFingerprint != pNode.getPackageFingerprint()) {
                throw new AconfigStorageReadException(
                        AconfigStorageReadException.ERROR_FILE_FINGERPRINT_MISMATCH,
                        "The fingerprint provided for the Aconfig package does not match the"
                                + " fingerprint of the package found on the device.");
            }

            return new AconfigPackageLocal(
                    fileProvider.getFlagValueList(container), pNode.getBooleanStartIndex());

        } catch (AconfigStorageException e) {
            throw new AconfigStorageReadException(e.getErrorCode(), e);
        } finally {
            StrictMode.setThreadPolicy(oldPolicy);
        }
    }

    /**
     * Retrieves the value of a boolean flag using its index.
     *
     * <p>This method is intended for internal use only and may be changed or removed without
     * notice.
     *
     * <p>This method retrieves the value of a flag within the loaded Aconfig package using its
     * index. The index is generated at build time and may vary between builds.
     *
     * <p>To ensure you are using the correct index, verify that the package's fingerprint matches
     * the expected fingerprint before calling this method. If the fingerprints do not match, use
     * {@link #getBooleanFlagValue(String, boolean)} instead.
     *
     * @param index The index of the flag within the package.
     * @return The boolean value of the flag.
     * @hide
     */
    @UnsupportedAppUsage
    public boolean getBooleanFlagValue(int index) {
        return mFlagValueList.getBoolean(index + mPackageBooleanStartOffset);
    }
}
