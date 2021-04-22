pipeline {
    agent none

    stages {
        stage('shell') {
            agent { label 'script' }
            steps {
                sh script: './qa-test --shell', label: 'shell scripts lint'
            }
            post {
                always {
                    // linters results
                    recordIssues enabledForFailure: true, id: 'shellcheck', failOnError: true, sourceCodeEncoding: 'UTF-8',
                                 tool: checkStyle(pattern: 'shellcheck/*.log', reportEncoding: 'UTF-8')
                }
            }
        }
        stage('rudder-pkg') {
            agent { label 'script' }
            steps {
                dir ('relay/sources') {
                    sh script: 'make check', label: 'rudder-pkg tests'
                }
            }
            post {
                always {
                    // linters results
                    recordIssues enabledForFailure: true, id: 'rudder-pkg', failOnError: true, sourceDirectory: 'relay/sources', sourceCodeEncoding: 'UTF-8',
                                 tool: pyLint(pattern: 'relay/sources/pylint.log', reportEncoding: 'UTF-8')
                }
            }
        }
        stage('webapp') {
            agent { label 'scala' }
            steps {
                dir('webapp/sources') {
                    withMaven() {
                        sh script: 'mvn clean install -Dmaven.test.postgres=false', label: "webapp tests"
                    }
                }
                sh script: 'webapp/sources/rudder/rudder-core/src/test/resources/hooks.d/test-hooks.sh', label: "hooks tests"
            }
            post {
                always {
                    // collect test results
                    junit 'webapp/sources/**/target/surefire-reports/*.xml'
                }
            }
        }
        stage('relayd') {
            agent { label 'rust' }
            steps {
                dir('relay/sources/relayd') {
                    sh script: 'make check', label: 'relayd tests'
                }
            }
            post {
                always {
                    // linters results
                    recordIssues enabledForFailure: true, id: 'relayd', name: 'relayd', failOnError: true, sourceDirectory: 'relay/sources/relayd', sourceCodeEncoding: 'UTF-8',
                                 tool: cargo(pattern: 'relay/sources/relayd/target/cargo-clippy.json', reportEncoding: 'UTF-8')
                }
            }
        }
        stage('language') {
            agent { label 'rust' }
            steps {
                dir('rudder-lang') {
                    sh script: 'make check', label: 'language tests'
                }
            }
            post {
                always {
                    // linters results
                    recordIssues enabledForFailure: true, id: 'language', name: 'language', failOnError: true, sourceDirectory: 'rudder-lang', sourceCodeEncoding: 'UTF-8',
                                 tool: cargo(pattern: 'rudder-lang/target/cargo-clippy.json', reportEncoding: 'UTF-8')
                }
            }
        }
    }


}
