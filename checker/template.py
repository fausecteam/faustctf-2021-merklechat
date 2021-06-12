#!/usr/bin/env python3

from ctf_gameserver import checkerlib

import utils


class TemplateChecker(checkerlib.BaseChecker):

    def place_flag(self, tick):
        # TODO: Implement
        return checkerlib.CheckResult.OK

    def check_service(self):
        # TODO: Implement (maybe use `utils.generate_message()`)
        return checkerlib.CheckResult.OK

    def check_flag(self, tick):
        # TODO: Implement
        return checkerlib.CheckResult.OK


if __name__ == '__main__':

    checkerlib.run_check(TemplateChecker)
