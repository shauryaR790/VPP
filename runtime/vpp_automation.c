// Automation primitives for v1.0.5 (process argv, env, directories).
// Included into the linked runtime via runtime_c_source().

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if !defined(_WIN32)
#include <unistd.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>
#include <errno.h>
#else
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#endif

static VppString* g_cmd_stdout = NULL;
static VppString* g_cmd_stderr = NULL;

static void vpp_set_cmd_output(VppString* out, VppString* err) {
    if (g_cmd_stdout) vpp_string_release(g_cmd_stdout);
    if (g_cmd_stderr) vpp_string_release(g_cmd_stderr);
    g_cmd_stdout = out ? vpp_string_retain(out) : vpp_string_new("");
    g_cmd_stderr = err ? vpp_string_retain(err) : vpp_string_new("");
}

static VppString* vpp_array_string_at(VppArray* arr, int64_t idx) {
    if (!arr || idx < 0 || idx >= arr->len) return NULL;
    VppString** slot = (VppString**)vpp_array_index_ptr(arr, idx);
    return slot ? *slot : NULL;
}

static char* vpp_read_pipe_to_string(int fd) {
#if !defined(_WIN32)
    char buf[4096];
    size_t cap = 4096;
    size_t len = 0;
    char* data = (char*)malloc(cap);
    if (!data) return NULL;
    for (;;) {
        ssize_t n = read(fd, buf, sizeof(buf));
        if (n <= 0) break;
        if (len + (size_t)n + 1 > cap) {
            cap = (len + (size_t)n + 1) * 2;
            char* next = (char*)realloc(data, cap);
            if (!next) break;
            data = next;
        }
        memcpy(data + len, buf, (size_t)n);
        len += (size_t)n;
    }
    data[len] = '\0';
    return data;
#else
    (void)fd;
    return vpp_strdup("");
#endif
}

#if !defined(_WIN32)
int64_t vpp_command_run(VppString* program, VppArray* args, VppString* cwd, int64_t timeout_ms) {
    (void)timeout_ms; /* v1.0.5: timeout wired in interpreter; native follow-up */
    const char* prog = vpp_string_cstr(program);
    int64_t argc = vpp_array_len(args);
    char** argv = (char**)calloc((size_t)argc + 2, sizeof(char*));
    if (!argv) return -1;
    argv[0] = vpp_strdup(prog);
    for (int64_t i = 0; i < argc; i++) {
        VppString* s = vpp_array_string_at(args, i);
        argv[i + 1] = vpp_strdup(vpp_string_cstr(s));
    }
    argv[argc + 1] = NULL;

    int outpipe[2];
    int errpipe[2];
    if (pipe(outpipe) != 0 || pipe(errpipe) != 0) {
        for (int64_t i = 0; i <= argc; i++) free(argv[i]);
        free(argv);
        return -1;
    }

    pid_t pid = fork();
    if (pid < 0) {
        close(outpipe[0]); close(outpipe[1]); close(errpipe[0]); close(errpipe[1]);
        for (int64_t i = 0; i <= argc; i++) free(argv[i]);
        free(argv);
        return -1;
    }
    if (pid == 0) {
        close(outpipe[0]);
        close(errpipe[0]);
        dup2(outpipe[1], STDOUT_FILENO);
        dup2(errpipe[1], STDERR_FILENO);
        close(outpipe[1]);
        close(errpipe[1]);
        const char* cd = vpp_string_cstr(cwd);
        if (cd && cd[0] != '\0') {
            chdir(cd);
        }
        execvp(prog, argv);
        _exit(127);
    }

    close(outpipe[1]);
    close(errpipe[1]);
    char* out_text = vpp_read_pipe_to_string(outpipe[0]);
    char* err_text = vpp_read_pipe_to_string(errpipe[0]);
    close(outpipe[0]);
    close(errpipe[0]);

    int status = 0;
    waitpid(pid, &status, 0);

    for (int64_t i = 0; i <= argc; i++) free(argv[i]);
    free(argv);

    vpp_set_cmd_output(vpp_string_new(out_text ? out_text : ""), vpp_string_new(err_text ? err_text : ""));
    free(out_text);
    free(err_text);

    if (WIFEXITED(status)) return (int64_t)WEXITSTATUS(status);
    return -1;
}
#else
int64_t vpp_command_run(VppString* program, VppArray* args, VppString* cwd, int64_t timeout_ms) {
    (void)timeout_ms;
    const char* prog = vpp_string_cstr(program);
    int64_t argc = vpp_array_len(args);

    SECURITY_ATTRIBUTES sa = {sizeof(SECURITY_ATTRIBUTES), NULL, TRUE};
    HANDLE out_r = NULL, out_w = NULL, err_r = NULL, err_w = NULL;
    if (!CreatePipe(&out_r, &out_w, &sa, 0) || !CreatePipe(&err_r, &err_w, &sa, 0)) {
        return -1;
    }
    SetHandleInformation(out_r, HANDLE_FLAG_INHERIT, 0);
    SetHandleInformation(err_r, HANDLE_FLAG_INHERIT, 0);

    char cmdline[8192];
    size_t pos = 0;
    cmdline[0] = '\0';
    {
        const char* piece = prog;
        size_t plen = strlen(piece);
        if (plen + 1 < sizeof(cmdline)) {
            memcpy(cmdline, piece, plen);
            pos = plen;
            cmdline[pos] = '\0';
        }
    }
    for (int64_t i = 0; i < argc; i++) {
        VppString* s = vpp_array_string_at(args, i);
        const char* piece = vpp_string_cstr(s);
        size_t plen = strlen(piece);
        if (pos + plen + 2 >= sizeof(cmdline)) break;
        cmdline[pos++] = ' ';
        memcpy(cmdline + pos, piece, plen);
        pos += plen;
        cmdline[pos] = '\0';
    }

    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    ZeroMemory(&si, sizeof(si));
    ZeroMemory(&pi, sizeof(pi));
    si.cb = sizeof(si);
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = out_w;
    si.hStdError = err_w;
    si.hStdInput = GetStdHandle(STD_INPUT_HANDLE);

    const char* cd = vpp_string_cstr(cwd);
    char* mutable_cmd = vpp_strdup(cmdline);
    BOOL ok = CreateProcessA(
        NULL,
        mutable_cmd,
        NULL,
        NULL,
        TRUE,
        CREATE_NO_WINDOW,
        NULL,
        (cd && cd[0] != '\0') ? cd : NULL,
        &si,
        &pi
    );
    free(mutable_cmd);
    CloseHandle(out_w);
    CloseHandle(err_w);
    if (!ok) {
        CloseHandle(out_r);
        CloseHandle(err_r);
        return -1;
    }

    WaitForSingleObject(pi.hProcess, INFINITE);
    DWORD exit_code = 1;
    GetExitCodeProcess(pi.hProcess, &exit_code);
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);

    char out_buf[65536];
    char err_buf[65536];
    DWORD out_read = 0, err_read = 0;
    ReadFile(out_r, out_buf, sizeof(out_buf) - 1, &out_read, NULL);
    ReadFile(err_r, err_buf, sizeof(err_buf) - 1, &err_read, NULL);
    out_buf[out_read] = '\0';
    err_buf[err_read] = '\0';
    CloseHandle(out_r);
    CloseHandle(err_r);

    vpp_set_cmd_output(vpp_string_new(out_buf), vpp_string_new(err_buf));
    return (int64_t)exit_code;
}
#endif

VppString* vpp_command_stdout(void) {
    return g_cmd_stdout ? vpp_string_retain(g_cmd_stdout) : vpp_string_new("");
}

VppString* vpp_command_stderr(void) {
    return g_cmd_stderr ? vpp_string_retain(g_cmd_stderr) : vpp_string_new("");
}

VppString* vpp_env_get(VppString* key) {
    const char* k = vpp_string_cstr(key);
#if defined(_WIN32)
    char buf[32768];
    DWORD n = GetEnvironmentVariableA(k, buf, (DWORD)sizeof(buf));
    if (n == 0 || n >= sizeof(buf)) return vpp_string_new("");
    return vpp_string_new(buf);
#else
    const char* v = getenv(k);
    return vpp_string_new(v ? v : "");
#endif
}

void vpp_env_set(VppString* key, VppString* value) {
    const char* k = vpp_string_cstr(key);
    const char* v = vpp_string_cstr(value);
#if defined(_WIN32)
    SetEnvironmentVariableA(k, v);
#else
    setenv(k, v, 1);
#endif
}

int32_t vpp_dir_exists(VppString* path) {
    const char* p = vpp_string_cstr(path);
#if defined(_WIN32)
    DWORD attr = GetFileAttributesA(p);
    return (attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY)) ? 1 : 0;
#else
    struct stat st;
    return (stat(p, &st) == 0 && S_ISDIR(st.st_mode)) ? 1 : 0;
#endif
}

void vpp_dir_create(VppString* path) {
    const char* p = vpp_string_cstr(path);
#if defined(_WIN32)
    CreateDirectoryA(p, NULL);
#else
    mkdir(p, 0755);
#endif
}

VppArray* vpp_dir_list(VppString* path) {
    const char* p = vpp_string_cstr(path);
#if defined(_WIN32)
    char pattern[4096];
    snprintf(pattern, sizeof(pattern), "%s\\*", p);
    WIN32_FIND_DATAA fd;
    HANDLE h = FindFirstFileA(pattern, &fd);
    if (h == INVALID_HANDLE_VALUE) {
        return vpp_make_array(0, (int64_t)sizeof(VppString*));
    }
    VppString* names[512];
    int count = 0;
    do {
        if (strcmp(fd.cFileName, ".") == 0 || strcmp(fd.cFileName, "..") == 0) continue;
        if (count < 512) names[count++] = vpp_string_new(fd.cFileName);
    } while (FindNextFileA(h, &fd) && count < 512);
    FindClose(h);
    VppArray* arr = vpp_make_array(count, (int64_t)sizeof(VppString*));
    for (int i = 0; i < count; i++) {
        VppString** slot = (VppString**)vpp_array_index_ptr(arr, i);
        *slot = names[i];
    }
    return arr;
#else
    DIR* dir = opendir(p);
    if (!dir) return vpp_make_array(0, (int64_t)sizeof(VppString*));
    VppString* names[512];
    int count = 0;
    struct dirent* ent;
    while ((ent = readdir(dir)) != NULL && count < 512) {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0) continue;
        names[count++] = vpp_string_new(ent->d_name);
    }
    closedir(dir);
    VppArray* arr = vpp_make_array(count, (int64_t)sizeof(VppString*));
    for (int i = 0; i < count; i++) {
        VppString** slot = (VppString**)vpp_array_index_ptr(arr, i);
        *slot = names[i];
    }
    return arr;
#endif
}

void vpp_log_line(VppString* level, VppString* message) {
    fprintf(stderr, "[%s] %s\n", vpp_string_cstr(level), vpp_string_cstr(message));
    fflush(stderr);
}

typedef struct {
    VppString* name;
    VppString* program;
    VppArray* args;
    VppString* cwd;
    int64_t timeout_ms;
} VppWorkflowTask;

int64_t vpp_workflow_parallel_tasks(VppArray* tasks) {
    if (!tasks) return 0;
    int64_t n = vpp_array_len(tasks);
    int64_t failed = 0;
    for (int64_t i = 0; i < n; i++) {
        VppWorkflowTask** slot = (VppWorkflowTask**)vpp_array_index_ptr(tasks, i);
        if (!slot || !*slot) {
            failed = 1;
            continue;
        }
        VppWorkflowTask* t = *slot;
        int64_t code = vpp_command_run(t->program, t->args, t->cwd, t->timeout_ms);
        if (code != 0) {
            failed = 1;
        }
    }
    return failed;
}
