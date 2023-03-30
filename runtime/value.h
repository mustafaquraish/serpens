#pragma once

#include <cstdint>
#include <memory>
#include <vector>
#include <string>

using namespace std;

typedef int32_t i32;
typedef int64_t i64;
typedef uint32_t u32;
typedef uint64_t u64;
typedef double f64;

#define Ref shared_ptr
#define make make_shared

enum class ValueKind{
    Nothing,
    Integer,
    String,
    Float,
    BuiltInFunction,
    Iterator,
    Range
};

struct Value;

struct BuiltInFunction {
    const char *name;
    Ref<Value> (*func)(vector<Ref<Value>>, const char *);
};

struct Iterator {
    virtual ~Iterator() {}
    virtual bool has_next() = 0;
    virtual Ref<Value> next() = 0;
};

struct RangeValue {
    i64 start;
    i64 end;
};

struct Value {
    ValueKind kind;
    union {
        i64 as_int;
        f64 as_float;
        std::string *as_string;
        BuiltInFunction as_builtin;
        Iterator *as_iter;
        RangeValue as_range;
    };

    Value(ValueKind kind): kind(kind) {}
    ~Value();

    static Ref<Value> from_int(i64 value);
    static Ref<Value> from_string(string value);
    static Ref<Value> from_float(f64 value);
    static Ref<Value> from_builtin(const char *name, Ref<Value> (*func)(vector<Ref<Value>>, const char *));
    static Ref<Value> from_iterator(Iterator *iter);
    static Ref<Value> from_range(i64 start, i64 end);

    Ref<Value> add(Ref<Value> other, const char *loc);
    Ref<Value> sub(Ref<Value> other, const char *loc);
    Ref<Value> mul(Ref<Value> other, const char *loc);
    Ref<Value> div(Ref<Value> other, const char *loc);

    Ref<Value> iter(const char *loc);
};

extern Ref<Value> Nothing;

Ref<Value> print(vector<Ref<Value>> args, const char *loc);


