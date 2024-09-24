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

package android.provider;

import android.annotation.FlaggedApi;
import android.annotation.NonNull;
import android.annotation.SystemApi;
import android.provider.flags.Flags;
import android.util.Log;
import java.util.Map;

/** @hide */
@SystemApi
@FlaggedApi(Flags.FLAG_STAGE_FLAGS_FOR_BUILD)
public final class StageOtaFlags {
  private static String LOG_TAG = "StageOtaFlags";

  private StageOtaFlags() {}

  /**
   * Stage aconfig flags to be applied when booting into {@code buildId}.
   *
   * <p>Only a single {@code buildId} and its corresponding flags are stored at
   * once. Every invocation of this method will overwrite whatever mapping was
   * previously stored.
   *
   * It is an implementation error to call this if the storage is not
   * initialized and ready to receive writes. Callers must ensure that it is
   * available before invoking.
   *
   * TODO(b/361783454): create an isStorageAvailable API and mention it in this
   * docstring.
   *
   * @param flags a map from {@code <packagename>.<flagname>} to flag values
   * @param buildId when the device boots into buildId, it will apply {@code flags}
   * @throws IllegalStateException if the storage is not ready to receive writes
   *
   * @hide
   */
  @SystemApi
  @FlaggedApi(Flags.FLAG_STAGE_FLAGS_FOR_BUILD)
  public static void stageBooleanAconfigFlagsForBuild(
      @NonNull Map<String, Boolean> flags, @NonNull String buildId) {
    int flagCount = flags.size();
    Log.d(LOG_TAG, "stageFlagsForBuild invoked for " + flagCount + " flags");
  }
}
