import {fake_matrix_api_handler} from './matrix-fake-api';
import {login} from "./utils";
describe('LoginPage', () => {

    beforeEach(function () {
        // We use cy.visit({onBeforeLoad: ...}) to stub
        // window.fetch before any app code runs
        cy.visit('/', {
            onBeforeLoad(win) {
                let fetch = win.fetch;
                cy.stub(win, 'fetch')
                    .callsFake(args => fake_matrix_api_handler(args, fetch, win));
            },
        })
    })

    it('Does login', () => {
        login()
        // First show spinner
        cy.get('svg#loading').should('be.visible');
        // Now it should be gone and instead we see the roomlist
        cy.get('svg#loading').should('not.be.visible');
        cy.get('ul.scrollable').should('be.visible');
    })
})
