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

#pragma once

namespace android {
namespace aconfigd {

/// start aconfigd scoket
int aconfigd_mainline_start_socket();

/// initialize storage files for bootstraped apexes
int aconfigd_mainline_bootstrap_init();

/// initialize storage files for all apexes
int aconfigd_mainline_init();

} // namespace aconfigd
} // namespace android
