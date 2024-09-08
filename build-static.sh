#!/bin/sh

elm make src/pages/Login.elm --output=static/login.html $@
elm make src/pages/Register.elm --output=static/register.html $@
elm make src/pages/Activate.elm --output=static/activate.html $@
