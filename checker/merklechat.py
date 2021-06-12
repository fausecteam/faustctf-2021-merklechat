#!/usr/bin/env python3

from ctf_gameserver import checkerlib
from selenium import webdriver
import selenium.common.exceptions
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.support.ui import WebDriverWait
import utils
from pprint import pprint
from time import sleep
import secrets
import json
import tempfile
import logging
import socket
try:
    from sh import git
except:
    def git(*args):
        return "package not installed"
#import IPython

from selenium.webdriver.chrome.options import Options

chrome_options = Options()
chrome_options.add_argument("--headless")
chrome_options.add_argument("--disable-extensions")
#chrome_options.add_argument("--no-sandbox")

FIRSTNAMES = (
    'P1r4t',
    'H4xXx0r',
    'n33rd',
)
LASTNAMES = FIRSTNAMES

class MerkleChecker(checkerlib.BaseChecker):
    def __init__(self, *args):
        logging.info("Version 0.f (..)")
        checkerlib.BaseChecker.__init__(self, *args)
        (seleniumlog, self._filename) = tempfile.mkstemp()
        try:
            self._driver = webdriver.Chrome(options=chrome_options,
                                            service_args=[
                                                f'--log-path={self._filename}'
                                            ])
            self._driver.set_page_load_timeout(30)
        except:
            with open(self._filename) as fd:
                logging.exception("%s", fd.read())
            raise
        self._accounts = []


    def __del__(self):
        if hasattr(self, '_driver'):
            self._driver.close()
            self._driver.quit()


    def _register(self):
        driver = self._driver
        driver.get(f'http://[{self.ip}]:4296')
        for field in ('uuid', 'secretkeys'):
            accountdata = driver.execute_script("window.localStorage.removeItem(arguments[0])", field)
        driver.get(f'http://[{self.ip}]:4296')

        WebDriverWait(driver, 5, 0.1).until(
            lambda driver: driver.find_element_by_id('modal').find_element_by_id('register-username')
        )
        name = f'{secrets.choice(FIRSTNAMES)} {secrets.choice(FIRSTNAMES)} {secrets.token_urlsafe(8)}'
        driver.find_element_by_id('register-username').send_keys(name)
        driver.find_element_by_id('register-username').send_keys(Keys.RETURN)
        WebDriverWait(driver, 100).until(
            lambda driver: 'uuid' in driver.execute_script("return window.localStorage;").keys()
        )
        accountdata = driver.execute_script("return window.localStorage;")
        return {'secretkeys': accountdata['secretkeys'],
                'uuid': accountdata['uuid'],
                'name': name,
                }


    def _login(self, account):
        self._accounts.append(account)
        driver = self._driver
        driver.get(f'http://[{self.ip}]:4296')
        for field in ('uuid', 'secretkeys'):
            accountdata = driver.execute_script("window.localStorage.removeItem(arguments[0])", field)
        for field in ('uuid', 'secretkeys'):
            accountdata = driver.execute_script("window.localStorage.setItem(arguments[0], arguments[1])", field, account[field])
        driver.get(f'http://[{self.ip}]:4296')
        WebDriverWait(driver, 5, 0.1).until(
            lambda driver: driver.execute_script("return window.account;")
        )

        logging.info("logged in: uuid: %s", driver.execute_script("return window.account.uuid;"))


    def _add_contact(self, uuid):
        driver = self._driver
        for elt in driver.find_elements_by_class_name('chat-subject'):
            if elt.text == 'Add Contact':
                elt.click()

        WebDriverWait(driver, 5, 0.1).until(
            lambda driver: driver.find_element_by_id('modal').find_element_by_id('addcontact-uuid')
        )
        driver.find_element_by_id('addcontact-uuid').send_keys(uuid)
        driver.find_element_by_id('addcontact-uuid').send_keys(Keys.RETURN)
        sleep(1)


    def _open_chat(self, name):
        driver = self._driver
        for elt in driver.find_elements_by_class_name('chat-subject'):
            if elt.text == name:
                elt.click()
                WebDriverWait(driver, 10, 0.1).until(
                    lambda driver: driver.find_element_by_id('input')
                )
                sleep(1)
                return


    def _send_message(self, message):
        driver = self._driver
        WebDriverWait(driver, 10, 0.1).until(
            lambda driver: driver.find_element_by_id('input')
        )
        inputf = driver.find_element_by_id('input')
        inputf.find_element_by_class_name('pure-input-1').send_keys(message)
        inputf.find_element_by_class_name('pure-input-1').send_keys(Keys.RETURN)
        WebDriverWait(driver, 100).until(
            lambda driver: '' == driver.find_element_by_id('input').find_element_by_class_name('pure-input-1').text
        )


    def _find_flag(self, flag, uuid=None):
        driver = self._driver
        driver.execute_script("window.account.update()")
        sleep(1.5)
        if uuid is not None:
            def _check(driver):
                for i in driver.find_elements_by_class_name('chat-item'):
                    try:
                        if i.get_attribute("data-uuid") == uuid:
                            return True
                    except:
                        logging.exception("|||| %s", repr(i))
                return False

            WebDriverWait(driver, 30, 0.1).until(
                #lambda driver: any([i.get_attribute("data-uuid") == uuid for i in driver.find_elements_by_class_name('chat-item')])
                _check
            )

        for elt in driver.find_elements_by_class_name('chat-item'):
            if (elt.text not in ['', 'Add Contact'] and
                (uuid is None or uuid == elt.get_attribute("data-uuid"))):
                for i in range(3):
                    try:
                        elt.click()
                        sleep(.2)
                        break
                    except selenium.common.exceptions.ElementClickInterceptedException:
                        continue
                else:
                    return False

                WebDriverWait(driver, 20, 0.1).until(
                    lambda driver: driver.find_element_by_class_name("chat-content").get_attribute('data-uuid') == elt.get_attribute("data-uuid")
                )
                logging.info("we have a body (%s)", driver.find_element_by_class_name("chat-content-body").text)

                WebDriverWait(driver, 20, 0.1).until(
                    lambda driver: flag in driver.find_element_by_class_name("chat-content-body").text
                )

                if flag in driver.find_element_by_class_name("chat-content-body").text:
                    return True
        return False


    def place_flag(self, tick):
        try:
            mytick = tick
            while mytick > 0 and tick - mytick < 5:
                account = checkerlib.load_state(f'account[{mytick}]')
                if account is not None:
                    logging.info("adding account: %s", account['uuid'])
                    self._accounts.append(account)
                mytick -= 1

            if len(self._accounts) == 0:
                self._accounts = [self._register()]

            self._account = self._register()
            checkerlib.store_state(f'account[{tick}]', self._account)
            flag = checkerlib.get_flag(tick)
            sender = secrets.choice(self._accounts)

            logging.info("account: %s, sender: %s", self._account['uuid'], sender['uuid'])
            checkerlib.store_state(f'sender[{tick}]', sender)
            self._login(sender)
            self._add_contact(self._account['uuid'])
            self._open_chat(self._account['name'])
            self._send_message(flag)

            self._login(self._account)
            try:
                if not self._find_flag(flag, sender['uuid']):
                    return checkerlib.CheckResult.FAULTY
            except selenium.common.exceptions.TimeoutException:
                logging.exception("place-flag-check")
                return checkerlib.CheckResult.FAULTY

            checkerlib.set_flagid(self._account['uuid'])
            return checkerlib.CheckResult.OK
        except selenium.common.exceptions.TimeoutException:
            logging.exception("place-flag")
            return checkerlib.CheckResult.DOWN
        except selenium.common.exceptions.WebDriverException:
            return checkerlib.CheckResult.DOWN
        except:
            # with open(self._filename) as fd:
            #     logging.exception("%s", fd.read())
            raise


    def check_service(self):
        try:
            for string in ["alert(window.localStorage);"
                          ,"';SELECT * from message;--"
                           ]:
                fromaccount = secrets.choice(self._accounts)
                toaccount = secrets.choice(self._accounts)
                logging.info("Sending `%s` from %s to %s", string, fromaccount['uuid'], toaccount['uuid'])

                self._login(fromaccount)
                self._add_contact(toaccount['uuid'])
                self._open_chat(toaccount['name'])
                self._send_message(string)
                self._login(toaccount)
                if not self._find_flag(string, fromaccount['uuid']):
                    return checkerlib.CheckResult.FAULTY

            return checkerlib.CheckResult.OK
        except selenium.common.exceptions.TimeoutException:
            logging.exception("check-service")
            return checkerlib.CheckResult.DOWN


    def check_flag(self, tick):
        try:
            account = checkerlib.load_state(f'account[{tick}]')
            sender =  checkerlib.load_state(f'sender[{tick}]')

            if account is None or sender is None:
                logging.info("NOTFOUND, account: %s, sender: %s", account, sender)
                return checkerlib.CheckResult.FLAG_NOT_FOUND

            self._login(account)
            try:
                if not self._find_flag(checkerlib.get_flag(tick), sender['uuid']):
                    return checkerlib.CheckResult.FLAG_NOT_FOUND
            except selenium.common.exceptions.TimeoutException:
                logging.exception("check-flag-check")
                return checkerlib.CheckResult.FLAG_NOT_FOUND


            return checkerlib.CheckResult.OK
        except selenium.common.exceptions.TimeoutException:
            logging.exception("check-flag")
            return checkerlib.CheckResult.DOWN
        except:
            # with open(self._filename) as fd:
            #     logging.exception("%s", fd.read())
            raise


if __name__ == '__main__':

    checkerlib.run_check(MerkleChecker)
