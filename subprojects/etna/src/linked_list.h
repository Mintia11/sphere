#pragma once

#include <stddef.h>

typedef struct etna_listnode etna_listnode_t;

typedef struct etna_listnode {
    etna_listnode_t* prev;
    etna_listnode_t* next;
} etna_listnode_t;

#ifndef container_of
#define etna_container_of(ptr, type, member) ((type*)((char*)(ptr) - offsetof(type, member)))
#else
#define etna_container_of container_of
#endif

#define ETNA_LISTNODE_INIT         \
    (etna_listnode_t) {            \
        .prev = NULL, .next = NULL \
    }

static inline void __list_add(etna_listnode_t* new_node, etna_listnode_t* prev,
                              etna_listnode_t* next) {
    next->prev = new_node;
    new_node->next = next;
    new_node->prev = prev;
    prev->next = new_node;
}

static inline void etna_list_add_tail(etna_listnode_t* head, etna_listnode_t* new_node) {
    __list_add(new_node, head->prev, head);
}

static inline void etna_list_del(etna_listnode_t* node) {
    node->next->prev = node->prev;
    node->prev->next = node->next;
    node->next = NULL;
    node->prev = NULL;
}

#define ETNA_LIST_FOR_EACH_ENTRY(pos, head, member)                                      \
    for (pos = container_of((head)->next, typeof(*pos), member); &pos->member != (head); \
         pos = container_of(pos->member.next, typeof(*pos), member))