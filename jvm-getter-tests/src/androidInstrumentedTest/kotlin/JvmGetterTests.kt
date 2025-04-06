// Copyright (c) 2025 Gobley Contributors.

package dev.gobley.jvmgetter.tests

import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.newSingleThreadContext
import kotlinx.coroutines.test.runTest
import org.junit.Test

class JvmGetterTests {
    @Test
    fun findJniGetCreatedJavaVms_returns_a_non_zero_value() {
        findJniGetCreatedJavaVms() shouldNotBe 0UL
    }

    @Test
    fun getJavaVm_returns_a_non_zero_value() {
        getJavaVm() shouldNotBe 0UL
    }

    @Test
    fun can_read_static_field_value_without_JNI_OnLoad() {
        getSimpleObjectFieldValueWithoutJniOnLoad() shouldBe SimpleObject.simpleValue
    }

    @Test
    @OptIn(ExperimentalCoroutinesApi::class, DelicateCoroutinesApi::class)
    fun can_read_static_field_value_without_JNI_OnLoad_from_a_new_Java_thread() = runTest {
        val singleThreadContext = newSingleThreadContext("New Single Thread Context")
        val singleThreadScope = CoroutineScope(singleThreadContext)
        val (threadName, actualValue) = singleThreadScope.async {
            Thread.currentThread().name to getSimpleObjectFieldValueWithoutJniOnLoad()
        }.await()
        threadName shouldNotBe Thread.currentThread().name
        actualValue shouldBe SimpleObject.simpleValue
    }

    @Test
    fun can_read_static_field_value_without_JNI_OnLoad_from_a_new_Rust_thread() = runTest {
        getSimpleObjectFieldValueWithoutJniOnLoadFromRustThread() shouldBe SimpleObject.simpleValue
    }
}