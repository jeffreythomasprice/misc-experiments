package com.jeffreythomasprice.experiments

import org.http4k.core.Filter
import org.http4k.core.Request
import org.http4k.core.Response
import org.http4k.core.Status
import kotlin.reflect.KClass
import kotlin.reflect.safeCast

class HttpStatusException(val status: Status, message: String? = null, cause: Throwable? = null) :
	Exception(message ?: status.description, cause)

inline fun <reified T : Any> Throwable.find(): T? =
	find(T::class)

fun <T : Any> Throwable.find(type: KClass<T>): T? =
	type.safeCast(this) ?: cause?.find(type)

inline fun <reified T : Throwable> exceptionFilter(crossinline f: (T) -> Response) =
	Filter { next ->
		{ request: Request ->
			try {
				next(request)
			} catch (e: Throwable) {
				val desiredException = e.find<T>()
				if (desiredException != null) {
					f(desiredException)
				} else {
					throw e
				}
			}
		}
	}