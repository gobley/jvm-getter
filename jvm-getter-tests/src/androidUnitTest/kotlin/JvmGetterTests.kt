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
    fun `findJniGetCreatedJavaVms returns a non-zero value`() {
        findJniGetCreatedJavaVms() shouldNotBe 0UL
    }

    @Test
    fun `getJavaVm returns a non zero value`() {
        getJavaVm() shouldNotBe 0UL
    }

    @Test
    fun `can read static field value without JNI_OnLoad`() {
        getSimpleObjectFieldValueWithoutJniOnLoad() shouldBe SimpleObject.simpleValue
    }

    @Test
    @OptIn(ExperimentalCoroutinesApi::class, DelicateCoroutinesApi::class)
    fun `can read static field value without JNI_OnLoad from a new Java thread`() = runTest {
        val singleThreadContext = newSingleThreadContext("New Single Thread Context")
        val singleThreadScope = CoroutineScope(singleThreadContext)
        val (threadName, actualValue) = singleThreadScope.async {
            Thread.currentThread().name to getSimpleObjectFieldValueWithoutJniOnLoad()
        }.await()
        threadName shouldNotBe Thread.currentThread().name
        actualValue shouldBe SimpleObject.simpleValue
    }

    @Test
    fun `can read static field value without JNI_OnLoad from a new Rust thread`() = runTest {
        getSimpleObjectFieldValueWithoutJniOnLoadFromRustThread() shouldBe SimpleObject.simpleValue
    }
}