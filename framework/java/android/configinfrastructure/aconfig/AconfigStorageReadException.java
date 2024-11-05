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

import android.annotation.FlaggedApi;
import android.annotation.NonNull;

/**
 * Exception thrown when an error occurs while reading from Aconfig Storage.
 *
 * <p>This exception indicates that there was a problem accessing or retrieving configuration data
 * from Aconfig Storage.
 */
@FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
public class AconfigStorageReadException extends RuntimeException {

    /**
     * Constructs a new {@code AconfigStorageReadException} with the specified detail message.
     *
     * @param msg The detail message for this exception.
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public AconfigStorageReadException(@NonNull String msg) {
        super(msg);
    }

    /**
     * Constructs a new {@code AconfigStorageReadException} with the specified detail message and
     * cause.
     *
     * @param msg The detail message for this exception.
     * @param cause The cause of this exception.
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public AconfigStorageReadException(@NonNull String msg, @NonNull Throwable cause) {
        super(msg, cause);
    }

    /**
     * Constructs a new {@code AconfigStorageReadException} with the specified cause.
     *
     * @param cause The cause of this exception.
     */
    @FlaggedApi(FLAG_NEW_STORAGE_PUBLIC_API)
    public AconfigStorageReadException(@NonNull Throwable cause) {
        super(cause);
    }
}
