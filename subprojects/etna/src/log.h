#pragma once

#include <stddef.h>
#include "linked_list.h"
#include "mutex.h"

typedef enum {
    ETNA_LOG_TRACE,
    ETNA_LOG_DEBUG,
    ETNA_LOG_INFO,
    ETNA_LOG_WARN,
    ETNA_LOG_ERROR,
    ETNA_LOG_FATAL,
} etna_log_severity_t;

typedef struct etna_log_scope {
    const char* name;
    struct etna_log_scope* parent;
} etna_log_scope_t;

typedef struct {
    void (*write)(etna_log_scope_t* scope, etna_log_severity_t severity, const char* str);
    etna_listnode_t list;
} etna_log_subscriber_t;

typedef struct {
    etna_mutex_t mtx;
    etna_listnode_t subscribers;
} etna_logger_t;

extern etna_logger_t* etna_global_logger;

#define ETNA_TRACE(scope, ...) \
    etna_log_message(etna_global_logger, scope, ETNA_LOG_TRACE, __VA_ARGS__)
#define ETNA_DEBUG(scope, ...) \
    etna_log_message(etna_global_logger, scope, ETNA_LOG_DEBUG, __VA_ARGS__)
#define ETNA_INFO(scope, ...) \
    etna_log_message(etna_global_logger, scope, ETNA_LOG_INFO, __VA_ARGS__)
#define ETNA_WARN(scope, ...) \
    etna_log_message(etna_global_logger, scope, ETNA_LOG_WARN, __VA_ARGS__)
#define ETNA_ERROR(scope, ...) \
    etna_log_message(etna_global_logger, scope, ETNA_LOG_ERROR, __VA_ARGS__)
#define ETNA_FATAL(scope, ...) \
    etna_log_message(etna_global_logger, scope, ETNA_LOG_FATAL, __VA_ARGS__)

void etna_logger_init_global();
void etna_logger_init(etna_logger_t* logger, etna_log_subscriber_t** subscribers,
                      size_t subscriber_count);
void etna_log_message(etna_logger_t* logger, etna_log_scope_t* scope, etna_log_severity_t severity,
                      const char* msg, ...) __attribute__((format(printf, 4, 5)));
etna_log_scope_t* etna_log_scope_new(const char* name, etna_log_scope_t* parent);

etna_log_subscriber_t* etna_log_stdio(etna_logger_t* logger);