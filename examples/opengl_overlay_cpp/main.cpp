// Copyright © SixtyFPS GmbH <info@sixtyfps.io>
// SPDX-License-Identifier: (GPL-3.0-only OR LicenseRef-SixtyFPS-commercial)

#include "scene.h"

#include <iostream>
#include <stdlib.h>
#include <stdio.h>

#include <GLES2/gl2.h>
#include <GLES2/gl2platform.h>

static GLint compile_shader(GLuint program, GLuint shader_type, const GLchar *const *source)
{
    auto shader_id = glCreateShader(shader_type);
    glShaderSource(shader_id, 1, source, nullptr);
    glCompileShader(shader_id);

    GLint compiled = 0;
    glGetShaderiv(shader_id, GL_COMPILE_STATUS, &compiled);
    if (!compiled) {
        GLint infoLen = 0;
        glGetShaderiv(shader_id, GL_INFO_LOG_LENGTH, &infoLen);
        if (infoLen > 1) {
            char *infoLog = reinterpret_cast<char *>(malloc(sizeof(char) * infoLen));
            glGetShaderInfoLog(shader_id, infoLen, NULL, infoLog);
            fprintf(stderr, "Error compiling %s shader:\n%s\n",
                    shader_type == GL_FRAGMENT_SHADER ? "fragment shader" : "vertex shader",
                    infoLog);
            free(infoLog);
        }
        glDeleteShader(shader_id);
        exit(1);
    }
    glAttachShader(program, shader_id);

    return shader_id;
}

class OpenGLAlphaOverlay
{
public:
    void operator()(sixtyfps::RenderingState state)
    {
        switch (state) {
        case sixtyfps::RenderingState::RenderingSetup:
            setup();
            break;
        case sixtyfps::RenderingState::BeforeRendering:
            break;
        case sixtyfps::RenderingState::AfterRendering:
            after();
            break;
        case sixtyfps::RenderingState::RenderingTeardown:
            teardown();
            break;
        }
    }

private:
    void setup()
    {
        program = glCreateProgram();

        const GLchar *const fragment_shader = "#version 100\n"
                                              "precision mediump float;\n"
                                              "void main() {\n"
                                              "    gl_FragColor = vec4(0.0, 0.0, 0.0, 0.5);\n"
                                              "}\n";

        const GLchar *const vertex_shader = "#version 100\n"
                                            "attribute vec2 position;\n"
                                            "void main() {\n"
                                            "    gl_Position = vec4(position - 0.5, 0.0, 1.0);\n"
                                            "}\n";

        auto fragment_shader_id = compile_shader(program, GL_FRAGMENT_SHADER, &fragment_shader);
        auto vertex_shader_id = compile_shader(program, GL_VERTEX_SHADER, &vertex_shader);

        GLint linked = 0;
        glLinkProgram(program);
        glGetProgramiv(program, GL_LINK_STATUS, &linked);

        if (!linked) {
            GLint infoLen = 0;
            glGetProgramiv(program, GL_INFO_LOG_LENGTH, &infoLen);
            if (infoLen > 1) {
                char *infoLog = reinterpret_cast<char *>(malloc(sizeof(char) * infoLen));
                glGetProgramInfoLog(program, infoLen, NULL, infoLog);
                fprintf(stderr, "Error linking shader:\n%s\n", infoLog);
                free(infoLog);
            }
            glDeleteProgram(program);
            exit(1);
        }
        glDetachShader(program, fragment_shader_id);
        glDetachShader(program, vertex_shader_id);

        position_location = glGetAttribLocation(program, "position");
    }

    void after()
    {
        glDisable(GL_BLEND);
        glUseProgram(program);
        const float vertices[] = { 0.5, 1.0, 0.0, 0.0, 1.0, 0.0 };
        glVertexAttribPointer(position_location, 2, GL_FLOAT, GL_FALSE, 0, vertices);
        glEnableVertexAttribArray(position_location);
        glDrawArrays(GL_TRIANGLES, 0, 3);
        glEnable(GL_BLEND);
    }

    void teardown() { glDeleteProgram(program); }

    GLuint program;
    GLuint position_location;
};

int main()
{
    auto app = App::create();

    app->window().set_rendering_notifier(OpenGLAlphaOverlay {});

    app->run();
}
