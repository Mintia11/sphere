#include "log.h"
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <vadefs.h>
#include "alloc.h"
#include "linked_list.h"
#include "mutex.h"

etna_logger_t* etna_global_logger = NULL;

void etna_logger_init_global() {
    etna_global_logger = ETNA_ALLOC_TYPE(NULL, etna_logger_t);

    etna_log_subscriber_t* subscribers[1] = {etna_log_stdio(etna_global_logger)};
    etna_logger_init(etna_global_logger, subscribers, 1);
}

void etna_logger_init(etna_logger_t* logger, etna_log_subscriber_t** subscribers,
                      size_t subscriber_count) {
    logger->mtx = ETNA_MUTEX_INIT;
    etna_list_init(&logger->subscribers);

    for (size_t i = 0; i < subscriber_count; i++) {
        etna_log_subscriber_t* subscriber = subscribers[i];
        etna_list_add(&logger->subscribers, &subscriber->list);
    }
}

void etna_log_message(etna_logger_t* logger, etna_log_scope_t* scope, etna_log_severity_t severity,
                      const char* msg, ...) {
    etna_mutex_lock(&logger->mtx);
    char buf[1024];

    va_list ap;
    va_start(ap, msg);
    vsnprintf(buf, 1024, msg, ap);
    va_end(ap);

    etna_log_subscriber_t* subscriber;
    ETNA_LIST_FOR_EACH_ENTRY(subscriber, &logger->subscribers, list) {
        subscriber->write(scope, severity, buf);
    }

    etna_mutex_unlock(&logger->mtx);
}

static void print_scope_chain(etna_log_scope_t* scope) {
    if (!scope)
        return;
    print_scope_chain(scope->parent);
    if (scope->parent)
        printf("::");
    printf("%s", scope->name);
}

void stdio_write(etna_log_scope_t* scope, etna_log_severity_t severity, const char* str) {
    const char* severity_to_str[] = {
        [ETNA_LOG_TRACE] = "trace", [ETNA_LOG_DEBUG] = "debug", [ETNA_LOG_INFO] = "info",
        [ETNA_LOG_WARN] = "warn ",  [ETNA_LOG_ERROR] = "error", [ETNA_LOG_FATAL] = "fatal",
    };

    const char* severity_to_color[] = {
        [ETNA_LOG_TRACE] = "30;1", [ETNA_LOG_DEBUG] = "34", [ETNA_LOG_INFO] = "32",
        [ETNA_LOG_WARN] = "33",    [ETNA_LOG_ERROR] = "31", [ETNA_LOG_FATAL] = "31;1",
    };

    printf("\033[%sm%s\033[0m | ", severity_to_color[severity], severity_to_str[severity]);

    print_scope_chain(scope);

    printf(" > %s", str);
}

etna_log_subscriber_t* etna_log_stdio(etna_logger_t* logger) {
    etna_log_subscriber_t* subscriber = ETNA_ALLOC_TYPE(logger, etna_log_subscriber_t);
    subscriber->write = stdio_write;
    return subscriber;
}

etna_log_scope_t* etna_log_scope_new(const char* name, etna_log_scope_t* parent) {
    etna_log_scope_t* scope = ETNA_ALLOC_TYPE(parent, etna_log_scope_t);
    scope->parent = parent;

    size_t name_len = strlen(name) + 1;
    char* name_buf = ETNA_ALLOC(scope, name_len);
    strncpy_s(name_buf, name_len, name, name_len);
    scope->name = name_buf;

    return scope;
}