#include "value.h"
#include <stdarg.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <iostream>

__attribute__((noreturn))
static void error(const char *loc, const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    fprintf(stderr, "%s: Error: ", loc);
    vfprintf(stderr, fmt, args);
    fprintf(stderr, "\n");
    va_end(args);
    exit(1);
}

Ref<Value> Nothing;

struct GlobalConstructor {
    GlobalConstructor() {
        Nothing = make<Value>(ValueKind::Nothing);
    }
} _{};

Ref<Value> Value::from_int(i64 value)  {
    auto val = make<Value>(ValueKind::Integer);
    val->as_int = value;
    return val;
}

Ref<Value> Value::from_string(string value)  {
    auto val = make<Value>(ValueKind::String);
    val->as_string = new string(value);
    return val;
}

Ref<Value> Value::from_float(f64 value)  {
    auto val = make<Value>(ValueKind::Float);
    val->as_float = value;
    return val;
}

Ref<Value> Value::from_iterator(Iterator *iter) {
    auto val = make<Value>(ValueKind::Iterator);
    val->as_iter = iter;
    return val;
}

Ref<Value> Value::from_range(i64 start, i64 end) {
    auto val = make<Value>(ValueKind::Range);
    val->as_range.start = start;
    val->as_range.end = end;
    return val;
}

Ref<Value> Value::from_builtin(const char *name, Ref<Value> (*func)(vector<Ref<Value>>, const char *)) {
    auto val = make<Value>(ValueKind::BuiltInFunction);
    val->as_builtin.name = name;
    val->as_builtin.func = func;
    return val;
}

struct StringIterator: Iterator {
    string str;
    int index;

    ~StringIterator() {}

    StringIterator(string *str): str(*str), index(0) {}

    bool has_next() {
        return index < str.size();
    }

    Ref<Value> next() {
        return Value::from_string(str.substr(index++, 1));
    }
};

struct RangeIterator: Iterator {
    i64 start;
    i64 end;
    i64 current;

    ~RangeIterator() {}

    RangeIterator(i64 start, i64 end): start(start), end(end), current(start) {}

    bool has_next() {
        return current < end;
    }

    Ref<Value> next() {
        return Value::from_int(current++);
    }
};


Ref<Value> Value::iter(const char *loc) {
    if (this->kind == ValueKind::String)
        return Value::from_iterator(new StringIterator(this->as_string));
    if (this->kind == ValueKind::Range)
        return Value::from_iterator(new RangeIterator(this->as_range.start, this->as_range.end));
    error(loc, "value is not iterable");
}

Ref<Value> Value::add(Ref<Value> other, const char *loc) {
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Integer)
        return Value::from_int(this->as_int + other->as_int);
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Float)
        return Value::from_float(this->as_int + other->as_float);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Integer)
        return Value::from_float(this->as_float + other->as_int);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Float)
        return Value::from_float(this->as_float + other->as_float);

    if (this->kind == ValueKind::String && other->kind == ValueKind::String)
        return Value::from_string(*this->as_string + *other->as_string);


    error(loc, "invalid operands to binary +");
}

Ref<Value> Value::sub(Ref<Value> other, const char *loc) {
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Integer)
        return Value::from_int(this->as_int - other->as_int);
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Float)
        return Value::from_float(this->as_int - other->as_float);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Integer)
        return Value::from_float(this->as_float - other->as_int);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Float)
        return Value::from_float(this->as_float - other->as_float);

    error(loc, "invalid operands to binary +");
}

Ref<Value> Value::mul(Ref<Value> other, const char *loc) {
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Integer)
        return Value::from_int(this->as_int * other->as_int);
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Float)
        return Value::from_float(this->as_int * other->as_float);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Integer)
        return Value::from_float(this->as_float * other->as_int);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Float)
        return Value::from_float(this->as_float * other->as_float);
    if (this->kind == ValueKind::String && other->kind == ValueKind::Integer) {
        string result;
        for (int i = 0; i < other->as_int; i++) {
            result += *this->as_string;
        }
        return Value::from_string(result);
    }

    error(loc, "invalid operands to binary +");
}

Ref<Value> Value::div(Ref<Value> other, const char *loc) {
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Integer)
        return Value::from_int(this->as_int / other->as_int);
    if (this->kind == ValueKind::Integer && other->kind == ValueKind::Float)
        return Value::from_float(this->as_int / other->as_float);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Integer)
        return Value::from_float(this->as_float / other->as_int);
    if (this->kind == ValueKind::Float && other->kind == ValueKind::Float)
        return Value::from_float(this->as_float / other->as_float);

    error(loc, "invalid operands to binary +");
}

Ref<Value> print(vector<Ref<Value>> args, const char *loc) {
    for (auto arg : args) {
        switch (arg->kind) {
        case ValueKind::Nothing:
            std::cout << "nothing";
            break;

        case ValueKind::Integer:
            std::cout << arg->as_int;
            break;

        case ValueKind::Float:
            std::cout << arg->as_float;
            break;

        case ValueKind::String:
            std::cout << *arg->as_string;
            break;

        case ValueKind::Range:
            std::cout << arg->as_range.start << ".." << arg->as_range.end;
            break;

        case ValueKind::Iterator:
            std::cout << "<iterator>";
            break;

        case ValueKind::BuiltInFunction:
            std::cout << "<builtin function: " << arg->as_builtin.name << ">";
            break;

        default:
            error(loc, "unsupported value kind in print");
            break;
        }
        std::cout << " ";
    }
    std::cout << std::endl;
    return Nothing;
}


Value::~Value() {
    switch (kind) {
    case ValueKind::String:
        delete this->as_string;
        break;

    case ValueKind::Iterator:
        delete this->as_iter;
        break;

    default:
        break;
    }
}

