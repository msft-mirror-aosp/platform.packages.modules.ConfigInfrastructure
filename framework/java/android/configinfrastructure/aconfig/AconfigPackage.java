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

import android.aconfig.storage.AconfigStorageException;
import android.aconfig.storage.FlagTable;
import android.aconfig.storage.FlagValueList;
import android.aconfig.storage.PackageTable;
import android.annotation.FlaggedApi;
import android.annotation.NonNull;
import android.os.StrictMode;

import java.io.Closeable;
import java.io.File;
import java.nio.MappedByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.Paths;
import java.nio.file.StandardOpenOption;

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
    private static final String MAP_PATH = "/metadata/aconfig/maps/";
    private static final String BOOT_PATH = "/metadata/aconfig/boot/";
    private static final String SYSTEM_MAP = "/metadata/aconfig/maps/system.package.map";
    private static final String PMAP_FILE_EXT = ".package.map";

    private FlagTable mFlagTable;
    private FlagValueList mFlagValueList;

    private int mPackageBooleanStartOffset = -1;
    private int mPackageId = -1;

    /**
     * This method will load a Aconfig Package from Aconfig Storage. If the package is not found, an
     * instance will be still created, but it will not be backed by a real Aconfig package in
     * storage.
     *
     * @param packageName name of the flag package
     * @return an instance of AconfigPackage
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public static @NonNull AconfigPackage load(@NonNull String packageName) {
        AconfigPackage aPackage = new AconfigPackage();
        aPackage.init(packageName);
        return aPackage;
    }

    /**
     * Get the value of a boolean flag.
     *
     * <p>This method retrieves the value of a boolean flag within the Aconfig Package. If the
     * instance is backed by a real Aconfig Package, and the flag is found in the Aconfig storage,
     * it returns the actual flag value. Otherwise, it returns the provided defaultValue.
     *
     * @param flagName flag name of the given flag (without a package name prefix)
     * @param defaultValue default value if the flag is not found or the AconfigPackage instance is
     *     not backed by real Aconfig package.
     * @return Boolean value indicates the flag value
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public boolean getBooleanFlagValue(@NonNull String flagName, boolean defaultValue) {
        // no such package in all containers
        if (mPackageId < 0) return defaultValue;
        FlagTable.Node fNode = mFlagTable.get(mPackageId, flagName);
        // no such flag in this package
        if (fNode == null) return defaultValue;
        int index = fNode.getFlagIndex() + mPackageBooleanStartOffset;
        return mFlagValueList.getBoolean(index);
    }

    private void init(String packageName) {
        StrictMode.ThreadPolicy oldPolicy = StrictMode.allowThreadDiskReads();

        try {
            // check system container first for optimization
            PackageTable pTable = null;
            PackageTable.Node pNode = null;
            // system map file does not exist on devices before A
            if (new File(SYSTEM_MAP).exists()) {
                pTable = PackageTable.fromBytes(mapStorageFile(SYSTEM_MAP));
                pNode = pTable.get(packageName);
            }
            String[] mapFiles = {};
            if (pNode == null) {
                mapFiles = new File(MAP_PATH).list();
                // return if the metadata folder doesn't exist
                if (mapFiles == null) return;
            }

            for (String file : mapFiles) {
                if (!file.endsWith(PMAP_FILE_EXT)) {
                    continue;
                }
                pTable = PackageTable.fromBytes(mapStorageFile(MAP_PATH + file));
                pNode = pTable.get(packageName);
                if (pNode != null) {
                    break;
                }
            }

            if (pNode == null) {
                // for the case package is not found in all container, return instead of throwing
                // error
                return;
            }

            String container = pTable.getHeader().getContainer();
            mFlagTable = FlagTable.fromBytes(mapStorageFile(MAP_PATH + container + ".flag.map"));
            mFlagValueList =
                    FlagValueList.fromBytes(mapStorageFile(BOOT_PATH + container + ".val"));
            mPackageBooleanStartOffset = pNode.getBooleanStartIndex();
            mPackageId = pNode.getPackageId();
        } catch (Exception e) {
            throw new AconfigStorageException("Fail to create AconfigPackage", e);
        } finally {
            StrictMode.setThreadPolicy(oldPolicy);
        }
    }

    // Map a storage file given file path
    private static MappedByteBuffer mapStorageFile(String file) {
        FileChannel channel = null;
        try {
            channel = FileChannel.open(Paths.get(file), StandardOpenOption.READ);
            return channel.map(FileChannel.MapMode.READ_ONLY, 0, channel.size());
        } catch (Exception e) {
            throw new AconfigStorageException(
                    String.format("Fail to mmap storage file %s", file), e);
        } finally {
            quietlyDispose(channel);
        }
    }

    private static void quietlyDispose(Closeable closable) {
        try {
            if (closable != null) {
                closable.close();
            }
        } catch (Exception e) {
            // no need to care, at least as of now
        }
    }
}
